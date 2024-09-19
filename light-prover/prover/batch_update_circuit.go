package prover

import (
	"fmt"
	merkle_tree "light/light-prover/merkle-tree"
	"light/light-prover/prover/poseidon"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/reilabs/gnark-lean-extractor/v2/abstractor"
)

type BatchUpdateCircuit struct {
	OldRoot             frontend.Variable `gnark:",public"`
	NewRoot             frontend.Variable `gnark:",public"`
	HashChainStartIndex frontend.Variable `gnark:",public"`
	LeavesHashchainHash frontend.Variable `gnark:",public"`

	Leaves       []frontend.Variable   `gnark:"input"`
	MerkleProofs [][]frontend.Variable `gnark:"input"`
	PathIndices  []frontend.Variable   `gnark:"input"`

	Height    uint32
	BatchSize uint32
}

func (circuit *BatchUpdateCircuit) Define(api frontend.API) error {
	calculatedHashchainHash := circuit.createHashChain(api, int(circuit.BatchSize), circuit.Leaves)
	api.AssertIsEqual(calculatedHashchainHash, circuit.LeavesHashchainHash)

	api.AssertIsEqual(circuit.HashChainStartIndex, 0)

	emptyLeaf := frontend.Variable(0)
	newRoot := circuit.OldRoot

	for i := 0; i < int(circuit.BatchSize); i++ {
		indexBits := api.ToBinary(circuit.PathIndices[i], int(circuit.Height))
		currentRoot := circuit.merkleRoot(api, circuit.Leaves[i], indexBits, circuit.MerkleProofs[i])
		api.AssertIsEqual(currentRoot, newRoot)
		newRoot = circuit.merkleRoot(api, emptyLeaf, indexBits, circuit.MerkleProofs[i])
	}

	api.AssertIsEqual(newRoot, circuit.NewRoot)

	return nil
}

func (circuit *BatchUpdateCircuit) merkleRoot(api frontend.API, leaf frontend.Variable, indexBits []frontend.Variable, siblings []frontend.Variable) frontend.Variable {
	currentHash := leaf

	for i := 0; i < int(circuit.Height); i++ {
		leftSibling := api.Select(indexBits[i], siblings[i], currentHash)
		rightSibling := api.Select(indexBits[i], currentHash, siblings[i])
		currentHash = abstractor.Call(api, poseidon.Poseidon2{In1: leftSibling, In2: rightSibling})
	}

	return currentHash
}

func (circuit *BatchUpdateCircuit) incrementBits(api frontend.API, bits []frontend.Variable) []frontend.Variable {
	carry := frontend.Variable(1)
	for i := 0; i < len(bits); i++ {
		newBit := api.Xor(bits[i], carry)
		carry = api.And(bits[i], carry)
		bits[i] = newBit
	}
	return bits
}

func (circuit *BatchUpdateCircuit) createHashChain(api frontend.API, length int, hashes []frontend.Variable) frontend.Variable {
	if length == 0 {
		return frontend.Variable(0)
	}

	hashChain := hashes[0]
	for i := 1; i < length; i++ {
		hashChain = abstractor.Call(api, poseidon.Poseidon2{In1: hashChain, In2: hashes[i]})
	}
	return hashChain
}

type BatchUpdateParameters struct {
	OldRoot             *big.Int
	NewRoot             *big.Int
	LeavesHashchainHash *big.Int
	Leaves              []*big.Int
	MerkleProofs        [][]big.Int
	PathIndices         []uint32
	HashChainStartIndex uint32
	Height              uint32
	BatchSize           uint32
	Tree                *merkle_tree.PoseidonTree
}

func (p *BatchUpdateParameters) TreeDepth() uint32 {
	if len(p.MerkleProofs) == 0 {
		return 0
	}
	return uint32(len(p.MerkleProofs[0]))
}

func (p *BatchUpdateParameters) ValidateShape() error {
	if len(p.Leaves) != int(p.BatchSize) {
		return fmt.Errorf("wrong number of leaves: %d, expected: %d", len(p.Leaves), p.BatchSize)
	}
	if len(p.MerkleProofs) != int(p.BatchSize) {
		return fmt.Errorf("wrong number of merkle proofs: %d", len(p.MerkleProofs))
	}
	if len(p.PathIndices) != int(p.BatchSize) {
		return fmt.Errorf("wrong number of path indices: %d", len(p.PathIndices))
	}
	for i, proof := range p.MerkleProofs {
		if len(proof) != int(p.Height) {
			return fmt.Errorf("wrong size of merkle proof for proof %d: %d", i, len(proof))
		}
	}
	return nil
}

func SetupBatchUpdate(height uint32, batchSize uint32) (*ProvingSystemV2, error) {
	fmt.Println("Setting up batch update")
	ccs, err := R1CSBatchUpdate(height, batchSize)
	if err != nil {
		return nil, err
	}
	pk, vk, err := groth16.Setup(ccs)
	if err != nil {
		return nil, err
	}
	return &ProvingSystemV2{
		TreeHeight:       height,
		BatchSize:        batchSize,
		ProvingKey:       pk,
		VerifyingKey:     vk,
		ConstraintSystem: ccs}, nil
}

func (ps *ProvingSystemV2) ProveBatchUpdate(params *BatchUpdateParameters) (*Proof, error) {
	if err := params.ValidateShape(); err != nil {
		return nil, err
	}

	oldRoot := frontend.Variable(params.OldRoot)
	newRoot := frontend.Variable(params.NewRoot)
	leavesHashchainHash := frontend.Variable(params.LeavesHashchainHash)
	hashChainStartIndex := frontend.Variable(params.HashChainStartIndex)

	leaves := make([]frontend.Variable, len(params.Leaves))
	pathIndices := make([]frontend.Variable, len(params.PathIndices))
	merkleProofs := make([][]frontend.Variable, len(params.MerkleProofs))

	for i := 0; i < len(params.Leaves); i++ {
		leaves[i] = frontend.Variable(params.Leaves[i])
		pathIndices[i] = frontend.Variable(params.PathIndices[i])
		merkleProofs[i] = make([]frontend.Variable, len(params.MerkleProofs[i]))
		for j := 0; j < len(params.MerkleProofs[i]); j++ {
			merkleProofs[i][j] = frontend.Variable(params.MerkleProofs[i][j])
		}
	}

	assignment := BatchUpdateCircuit{
		OldRoot:             oldRoot,
		NewRoot:             newRoot,
		LeavesHashchainHash: leavesHashchainHash,
		Leaves:              leaves,
		PathIndices:         pathIndices,
		MerkleProofs:        merkleProofs,
		HashChainStartIndex: hashChainStartIndex,
		Height:              ps.TreeHeight,
		BatchSize:           ps.BatchSize,
	}

	witness, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		return nil, fmt.Errorf("error creating witness: %v", err)
	}

	proof, err := groth16.Prove(ps.ConstraintSystem, ps.ProvingKey, witness)
	if err != nil {
		return nil, fmt.Errorf("error proving: %v", err)
	}

	return &Proof{proof}, nil
}

func R1CSBatchUpdate(height uint32, batchSize uint32) (constraint.ConstraintSystem, error) {
	leaves := make([]frontend.Variable, batchSize)
	pathIndices := make([]frontend.Variable, batchSize)
	merkleProofs := make([][]frontend.Variable, batchSize)

	for i := 0; i < int(batchSize); i++ {
		merkleProofs[i] = make([]frontend.Variable, height)
	}

	circuit := BatchUpdateCircuit{
		OldRoot:             frontend.Variable(0),
		NewRoot:             frontend.Variable(0),
		LeavesHashchainHash: frontend.Variable(0),
		Leaves:              leaves,
		PathIndices:         pathIndices,
		MerkleProofs:        merkleProofs,
		HashChainStartIndex: frontend.Variable(0),
		Height:              height,
		BatchSize:           batchSize,
	}

	return frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
}

func ImportBatchUpdateSetup(treeHeight uint32, batchSize uint32, pkPath string, vkPath string) (*ProvingSystemV2, error) {
	leaves := make([]frontend.Variable, batchSize)
	oldMerkleProofs := make([][]frontend.Variable, batchSize)
	newMerkleProofs := make([][]frontend.Variable, batchSize)

	for i := 0; i < int(batchSize); i++ {
		oldMerkleProofs[i] = make([]frontend.Variable, treeHeight)
		newMerkleProofs[i] = make([]frontend.Variable, treeHeight)
	}

	circuit := BatchUpdateCircuit{
		Height:       treeHeight,
		Leaves:       leaves,
		MerkleProofs: newMerkleProofs,
	}

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
