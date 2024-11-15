package prover

import (
	"fmt"
	"light/light-prover/logging"
	"light/light-prover/prover/poseidon"
	"math/big"

	merkletree "light/light-prover/merkle-tree"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/reilabs/gnark-lean-extractor/v2/abstractor"
)

type BatchAddressTreeAppendCircuit struct {
	PublicInputHash frontend.Variable `gnark:",public"`

	OldRoot       frontend.Variable `gnark:",private"`
	NewRoot       frontend.Variable `gnark:",private"`
	HashchainHash frontend.Variable `gnark:",private"`
	StartIndex    frontend.Variable `gnark:",private"`

	LowElementValues      []frontend.Variable   `gnark:",private"`
	LowElementNextIndices []frontend.Variable   `gnark:",private"`
	LowElementNextValues  []frontend.Variable   `gnark:",private"`
	LowElementIndices     []frontend.Variable   `gnark:",private"`
	LowElementProofs      [][]frontend.Variable `gnark:",private"`

	NewElementValues []frontend.Variable   `gnark:",private"`
	NewElementProofs [][]frontend.Variable `gnark:",private"`
	BatchSize        uint32
	TreeHeight       uint32
}

func (circuit *BatchAddressTreeAppendCircuit) Define(api frontend.API) error {
	currentRoot := circuit.OldRoot
	indexBits := api.ToBinary(circuit.StartIndex, int(circuit.TreeHeight))

	for i := uint32(0); i < circuit.BatchSize; i++ {
		api.Println("Processing element", i)
		api.Println("Current root", currentRoot)
		api.Println("Index bits", indexBits)
		api.Println("LeafLowerRangeValue[", i, "]", circuit.LowElementValues[i])
		api.Println("NextIndex[", i, "]", circuit.LowElementNextIndices[i])
		api.Println("LeafHigherRangeValue[", i, "]", circuit.LowElementNextValues[i])
		api.Println("Value[", i, "]", circuit.NewElementValues[i])

		oldLowLeafHash := abstractor.Call(api, LeafHashGadget{
			LeafLowerRangeValue:  circuit.LowElementValues[i],
			NextIndex:            circuit.LowElementNextIndices[i],
			LeafHigherRangeValue: circuit.LowElementNextValues[i],
			Value:                circuit.NewElementValues[i],
		})

		newLowLeafNextIndex := api.Add(circuit.StartIndex, i)
		lowLeafHash := abstractor.Call(api, poseidon.Poseidon3{
			In1: circuit.LowElementValues[i],
			In2: newLowLeafNextIndex,
			In3: circuit.NewElementValues[i],
		})
		pathIndexBits := api.ToBinary(circuit.LowElementIndices[i], int(circuit.TreeHeight))
		currentRoot = abstractor.Call(api, MerkleRootUpdateGadget{
			OldRoot:     currentRoot,
			OldLeaf:     oldLowLeafHash,
			NewLeaf:     lowLeafHash,
			PathIndex:   pathIndexBits,
			MerkleProof: circuit.LowElementProofs[i],
			Height:      int(circuit.TreeHeight),
		})

		// value = new value
		// next value is low leaf next value
		// next index is new value next index
		newLeafHash := abstractor.Call(api, poseidon.Poseidon3{
			In1: circuit.NewElementValues[i],
			In2: circuit.LowElementNextIndices[i],
			In3: circuit.LowElementNextValues[i],
		})

		currentRoot = abstractor.Call(api, MerkleRootUpdateGadget{
			OldRoot:     currentRoot,
			OldLeaf:     getZeroValue(0),
			NewLeaf:     newLeafHash,
			PathIndex:   indexBits,
			MerkleProof: circuit.NewElementProofs[i],
			Height:      int(circuit.TreeHeight),
		})

		indexBits = incrementBits(
			api,
			indexBits,
		)
	}

	api.AssertIsEqual(circuit.NewRoot, currentRoot)

	leavesHashChain := createHashChain(api, circuit.NewElementValues)
	api.AssertIsEqual(circuit.HashchainHash, leavesHashChain)

	publicInputsHashChain := circuit.computePublicInputHash(api)
	api.AssertIsEqual(circuit.PublicInputHash, publicInputsHashChain)

	return nil
}

func (circuit *BatchAddressTreeAppendCircuit) computePublicInputHash(api frontend.API) frontend.Variable {
	hashChainInputs := []frontend.Variable{
		circuit.OldRoot,
		circuit.NewRoot,
		circuit.HashchainHash,
		circuit.StartIndex,
	}

	return createHashChain(api, hashChainInputs)
}

func InitBatchAddressTreeAppendCircuit(treeHeight uint32, batchSize uint32) BatchAddressTreeAppendCircuit {
	logging.Logger().Info().
		Uint32("treeHeight", treeHeight).
		Uint32("batchSize", batchSize).
		Msg("Initializing batch address append circuit")

	lowElementValues := make([]frontend.Variable, batchSize)
	lowElementNextIndices := make([]frontend.Variable, batchSize)
	lowElementNextValues := make([]frontend.Variable, batchSize)
	lowElementIndices := make([]frontend.Variable, batchSize)
	lowElementProofs := make([][]frontend.Variable, batchSize)
	newElementValues := make([]frontend.Variable, batchSize)
	newElementProofs := make([][]frontend.Variable, batchSize)

	for i := uint32(0); i < batchSize; i++ {
		lowElementProofs[i] = make([]frontend.Variable, treeHeight)
		newElementProofs[i] = make([]frontend.Variable, treeHeight)
	}

	return BatchAddressTreeAppendCircuit{
		BatchSize:             batchSize,
		TreeHeight:            treeHeight,
		PublicInputHash:       frontend.Variable(0),
		OldRoot:               frontend.Variable(0),
		NewRoot:               frontend.Variable(0),
		HashchainHash:         frontend.Variable(0),
		StartIndex:            frontend.Variable(0),
		LowElementValues:      lowElementValues,
		LowElementNextIndices: lowElementNextIndices,
		LowElementNextValues:  lowElementNextValues,
		LowElementIndices:     lowElementIndices,
		LowElementProofs:      lowElementProofs,
		NewElementValues:      newElementValues,
		NewElementProofs:      newElementProofs,
	}
}

func (params *BatchAddressAppendParameters) CreateWitness() (*BatchAddressTreeAppendCircuit, error) {
	if params.BatchSize == 0 {
		return nil, fmt.Errorf("batch size cannot be 0")
	}
	if params.TreeHeight == 0 {
		return nil, fmt.Errorf("tree height cannot be 0")
	}

	logging.Logger().Debug().
		Interface("params", params).
		Msg("Creating witness with parameters")

	// Create circuit assignments
	circuit := &BatchAddressTreeAppendCircuit{
		BatchSize:             params.BatchSize,
		TreeHeight:            params.TreeHeight,
		PublicInputHash:       frontend.Variable(params.PublicInputHash),
		OldRoot:               frontend.Variable(params.OldRoot),
		NewRoot:               frontend.Variable(params.NewRoot),
		HashchainHash:         frontend.Variable(params.HashchainHash),
		StartIndex:            frontend.Variable(params.StartIndex),
		LowElementValues:      make([]frontend.Variable, params.BatchSize),
		LowElementNextIndices: make([]frontend.Variable, params.BatchSize),
		LowElementNextValues:  make([]frontend.Variable, params.BatchSize),
		LowElementIndices:     make([]frontend.Variable, params.BatchSize),
		NewElementValues:      make([]frontend.Variable, params.BatchSize),
		LowElementProofs:      make([][]frontend.Variable, params.BatchSize),
		NewElementProofs:      make([][]frontend.Variable, params.BatchSize),
	}

	// Initialize all arrays before filling
	for i := uint32(0); i < params.BatchSize; i++ {
		circuit.LowElementProofs[i] = make([]frontend.Variable, params.TreeHeight)
		circuit.NewElementProofs[i] = make([]frontend.Variable, params.TreeHeight)
	}

	// Fill in all values
	for i := uint32(0); i < params.BatchSize; i++ {
		circuit.LowElementValues[i] = frontend.Variable(&params.LowElementValues[i])
		circuit.LowElementNextIndices[i] = frontend.Variable(&params.LowElementNextIndices[i])
		circuit.LowElementNextValues[i] = frontend.Variable(&params.LowElementNextValues[i])
		circuit.LowElementIndices[i] = frontend.Variable(&params.LowElementIndices[i])
		circuit.NewElementValues[i] = frontend.Variable(&params.NewElementValues[i])

		for j := uint32(0); j < params.TreeHeight; j++ {
			if i < uint32(len(params.LowElementProofs)) {
				circuit.LowElementProofs[i][j] = frontend.Variable(&params.LowElementProofs[i][j])
			}
			if i < uint32(len(params.NewElementProofs)) {
				circuit.NewElementProofs[i][j] = frontend.Variable(&params.NewElementProofs[i][j])
			}
		}
	}

	// Log counts for debugging
	var totalVars int
	totalVars++ // PublicInputHash
	totalVars++ // OldRoot
	totalVars++ // NewRoot
	totalVars++ // HashchainHash
	totalVars++ // StartIndex
	totalVars += len(circuit.LowElementValues)
	totalVars += len(circuit.LowElementNextIndices)
	totalVars += len(circuit.LowElementNextValues)
	totalVars += len(circuit.LowElementIndices)
	totalVars += len(circuit.NewElementValues)
	for i := range circuit.LowElementProofs {
		totalVars += len(circuit.LowElementProofs[i])
	}
	for i := range circuit.NewElementProofs {
		totalVars += len(circuit.NewElementProofs[i])
	}

	logging.Logger().Debug().
		Int("totalVariables", totalVars).
		Int("batchSize", int(params.BatchSize)).
		Int("treeHeight", int(params.TreeHeight)).
		Msg("Created witness")

	return circuit, nil
}
func (p *BatchAddressAppendParameters) ValidateShape() error {
	expectedArrayLen := int(p.BatchSize)
	expectedProofLen := int(p.TreeHeight)

	if len(p.LowElementValues) != expectedArrayLen {
		return fmt.Errorf("wrong number of low element values: %d, expected: %d",
			len(p.LowElementValues), expectedArrayLen)
	}
	if len(p.LowElementIndices) != expectedArrayLen {
		return fmt.Errorf("wrong number of low element indices: %d, expected: %d",
			len(p.LowElementIndices), expectedArrayLen)
	}
	if len(p.LowElementNextIndices) != expectedArrayLen {
		return fmt.Errorf("wrong number of low element next indices: %d, expected: %d",
			len(p.LowElementNextIndices), expectedArrayLen)
	}
	if len(p.LowElementNextValues) != expectedArrayLen {
		return fmt.Errorf("wrong number of low element next values: %d, expected: %d",
			len(p.LowElementNextValues), expectedArrayLen)
	}
	if len(p.NewElementValues) != expectedArrayLen {
		return fmt.Errorf("wrong number of new element values: %d, expected: %d",
			len(p.NewElementValues), expectedArrayLen)
	}

	if len(p.LowElementProofs) != expectedArrayLen {
		return fmt.Errorf("wrong number of low element proofs: %d, expected: %d",
			len(p.LowElementProofs), expectedArrayLen)
	}
	if len(p.NewElementProofs) != expectedArrayLen {
		return fmt.Errorf("wrong number of new element proofs: %d, expected: %d",
			len(p.NewElementProofs), expectedArrayLen)
	}

	for i, proof := range p.LowElementProofs {
		if len(proof) != expectedProofLen {
			return fmt.Errorf("wrong proof length for LowElementProofs[%d]: got %d, expected %d",
				i, len(proof), expectedProofLen)
		}
	}
	for i, proof := range p.NewElementProofs {
		if len(proof) != expectedProofLen {
			return fmt.Errorf("wrong proof length for NewElementProofs[%d]: got %d, expected %d",
				i, len(proof), expectedProofLen)
		}
	}

	return nil
}

type BatchAddressAppendParameters struct {
	PublicInputHash *big.Int
	OldRoot         *big.Int
	NewRoot         *big.Int
	HashchainHash   *big.Int
	StartIndex      uint32

	LowElementValues      []big.Int
	LowElementIndices     []big.Int
	LowElementNextIndices []big.Int
	LowElementNextValues  []big.Int

	NewElementValues []big.Int

	LowElementProofs [][]big.Int
	NewElementProofs [][]big.Int

	TreeHeight uint32
	BatchSize  uint32
	Tree       *merkletree.IndexedMerkleTree
}

func SetupBatchAddressAppend(height uint32, batchSize uint32) (*ProvingSystemV2, error) {
	fmt.Println("Setting up batch update")
	ccs, err := R1CSBatchAddressAppend(height, batchSize)
	if err != nil {
		return nil, err
	}
	pk, vk, err := groth16.Setup(ccs)
	if err != nil {
		return nil, err
	}
	return &ProvingSystemV2{
		CircuitType:      BatchAddressAppendCircuitType,
		TreeHeight:       height,
		BatchSize:        batchSize,
		ProvingKey:       pk,
		VerifyingKey:     vk,
		ConstraintSystem: ccs}, nil
}

func R1CSBatchAddressAppend(height uint32, batchSize uint32) (constraint.ConstraintSystem, error) {
	circuit := InitBatchAddressTreeAppendCircuit(batchSize, height)
	return frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
}

func (ps *ProvingSystemV2) ProveBatchAddressAppend(params *BatchAddressAppendParameters) (*Proof, error) {
	if params == nil {
		panic("params cannot be nil")
	}
	if err := params.ValidateShape(); err != nil {
		return nil, err
	}

	assignment, err := params.CreateWitness()
	if err != nil {
		return nil, fmt.Errorf("error creating circuit: %v", err)
	}

	witness, err := frontend.NewWitness(assignment, ecc.BN254.ScalarField())
	if err != nil {
		return nil, fmt.Errorf("error creating witness: %v", err)
	}

	proof, err := groth16.Prove(ps.ConstraintSystem, ps.ProvingKey, witness)
	if err != nil {
		return nil, fmt.Errorf("error proving: %v", err)
	}

	return &Proof{proof}, nil
}

func ImportBatchAddressAppendSetup(treeHeight uint32, batchSize uint32, pkPath string, vkPath string) (*ProvingSystemV2, error) {
	circuit := InitBatchAddressTreeAppendCircuit(batchSize, treeHeight)

	fmt.Println("Compiling circuit")
	ccs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
	if err != nil {
		fmt.Println("Error compiling circuit")
		return nil, err
	} else {
		fmt.Println("Compiled circuit successfully")
	}

	pk, err := LoadProvingKey(pkPath)
	if err != nil {
		return nil, err
	}

	vk, err := LoadVerifyingKey(vkPath)
	if err != nil {
		return nil, err
	}

	return &ProvingSystemV2{
		TreeHeight:       treeHeight,
		BatchSize:        batchSize,
		ProvingKey:       pk,
		VerifyingKey:     vk,
		ConstraintSystem: ccs,
	}, nil
}
