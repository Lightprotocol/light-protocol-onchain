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
	PublicInputHash     frontend.Variable `gnark:",public"`
	OldRoot             frontend.Variable `gnark:",private"`
	NewRoot             frontend.Variable `gnark:",private"`
	LeavesHashchainHash frontend.Variable `gnark:",private"`

	TxHashes     []frontend.Variable   `gnark:"private"`
	Leaves       []frontend.Variable   `gnark:"private"`
	MerkleProofs [][]frontend.Variable `gnark:"private"`
	PathIndices  []frontend.Variable   `gnark:"private"`

	Height    uint32
	BatchSize uint32
}

func (circuit *BatchUpdateCircuit) Define(api frontend.API) error {

	hashChainInputs := make([]frontend.Variable, int(3))
	hashChainInputs[0] = circuit.OldRoot
	hashChainInputs[1] = circuit.NewRoot
	hashChainInputs[2] = circuit.LeavesHashchainHash
	publicInputsHashChain := createHashChain(api, int(3), hashChainInputs)
	api.AssertIsEqual(publicInputsHashChain, circuit.PublicInputHash)
	nullifiers := make([]frontend.Variable, int(circuit.BatchSize))
	for i := 0; i < int(circuit.BatchSize); i++ {
		nullifiers[i] = abstractor.Call(api, poseidon.Poseidon2{In1: circuit.Leaves[i], In2: circuit.TxHashes[i]})
	}

	nullifierHashChainHash := createHashChain(api, int(circuit.BatchSize), nullifiers)
	api.AssertIsEqual(nullifierHashChainHash, circuit.LeavesHashchainHash)

	newRoot := circuit.OldRoot

	for i := 0; i < int(circuit.BatchSize); i++ {
		newRoot = abstractor.Call(api, MerkleRootUpdateGadget{
			OldRoot:     newRoot,
			OldLeaf:     circuit.Leaves[i],
			NewLeaf:     nullifiers[i],
			PathIndex:   circuit.PathIndices[i],
			MerkleProof: circuit.MerkleProofs[i],
			Height:      int(circuit.Height),
		})
	}

	api.AssertIsEqual(newRoot, circuit.NewRoot)

	return nil
}

type BatchUpdateParameters struct {
	PublicInputHash     *big.Int
	OldRoot             *big.Int
	NewRoot             *big.Int
	TxHashes            []*big.Int
	LeavesHashchainHash *big.Int
	Leaves              []*big.Int
	MerkleProofs        [][]big.Int
	PathIndices         []uint32
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
	if len(p.TxHashes) != int(p.BatchSize) {
		return fmt.Errorf("wrong number of tx hashes: %d, expected: %d", len(p.TxHashes), p.BatchSize)
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

	publicInputHash := frontend.Variable(params.PublicInputHash)
	oldRoot := frontend.Variable(params.OldRoot)
	newRoot := frontend.Variable(params.NewRoot)
	leavesHashchainHash := frontend.Variable(params.LeavesHashchainHash)

	txHashes := make([]frontend.Variable, len(params.TxHashes))
	leaves := make([]frontend.Variable, len(params.Leaves))
	pathIndices := make([]frontend.Variable, len(params.PathIndices))
	merkleProofs := make([][]frontend.Variable, len(params.MerkleProofs))

	for i := 0; i < len(params.Leaves); i++ {
		leaves[i] = frontend.Variable(params.Leaves[i])
		txHashes[i] = frontend.Variable(params.TxHashes[i])
		pathIndices[i] = frontend.Variable(params.PathIndices[i])
		merkleProofs[i] = make([]frontend.Variable, len(params.MerkleProofs[i]))
		for j := 0; j < len(params.MerkleProofs[i]); j++ {
			merkleProofs[i][j] = frontend.Variable(params.MerkleProofs[i][j])
		}
	}

	assignment := BatchUpdateCircuit{
		PublicInputHash:     publicInputHash,
		OldRoot:             oldRoot,
		NewRoot:             newRoot,
		TxHashes:            txHashes,
		LeavesHashchainHash: leavesHashchainHash,
		Leaves:              leaves,
		PathIndices:         pathIndices,
		MerkleProofs:        merkleProofs,
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
	txHashes := make([]frontend.Variable, batchSize)
	pathIndices := make([]frontend.Variable, batchSize)
	merkleProofs := make([][]frontend.Variable, batchSize)

	for i := 0; i < int(batchSize); i++ {
		merkleProofs[i] = make([]frontend.Variable, height)
	}

	circuit := BatchUpdateCircuit{
		PublicInputHash:     frontend.Variable(0),
		OldRoot:             frontend.Variable(0),
		NewRoot:             frontend.Variable(0),
		TxHashes:            txHashes,
		LeavesHashchainHash: frontend.Variable(0),
		Leaves:              leaves,
		PathIndices:         pathIndices,
		MerkleProofs:        merkleProofs,
		Height:              height,
		BatchSize:           batchSize,
	}

	return frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
}

func ImportBatchUpdateSetup(treeHeight uint32, batchSize uint32, pkPath string, vkPath string) (*ProvingSystemV2, error) {
	leaves := make([]frontend.Variable, batchSize)
	txHashes := make([]frontend.Variable, batchSize)
	oldMerkleProofs := make([][]frontend.Variable, batchSize)
	newMerkleProofs := make([][]frontend.Variable, batchSize)

	for i := 0; i < int(batchSize); i++ {
		oldMerkleProofs[i] = make([]frontend.Variable, treeHeight)
		newMerkleProofs[i] = make([]frontend.Variable, treeHeight)
	}

	circuit := BatchUpdateCircuit{
		Height:              treeHeight,
		TxHashes:            txHashes,
		Leaves:              leaves,
		MerkleProofs:        newMerkleProofs,
		PathIndices:         make([]frontend.Variable, batchSize),
		OldRoot:             frontend.Variable(0),
		NewRoot:             frontend.Variable(0),
		LeavesHashchainHash: frontend.Variable(0),
		BatchSize:           batchSize,
		PublicInputHash:     frontend.Variable(0),
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
