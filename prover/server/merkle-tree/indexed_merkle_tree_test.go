package merkle_tree

import (
	"fmt"
	"math/big"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestIndexedMerkleTreeInit(t *testing.T) {
	expectedRoot := []byte{33, 133, 56, 184, 142, 166, 110, 161, 4, 140, 169, 247, 115, 33, 15, 181, 76, 89, 48, 126, 58, 86, 204, 81, 16, 121, 185, 77, 75, 152, 43, 15}

	tree, err := NewIndexedMerkleTree(26)
	require.NoError(t, err)

	err = tree.Init()
	require.NoError(t, err)

	root := tree.Tree.Root.Bytes()
	require.Equal(t, expectedRoot, root)

	require.Equal(t, uint32(0), tree.IndexArray.Get(0).Index)
	require.Equal(t, uint32(1), tree.IndexArray.Get(0).NextIndex)
	require.Equal(t, "0", tree.IndexArray.Get(0).Value.String())

	maxVal := new(big.Int).Sub(new(big.Int).Lsh(big.NewInt(1), 248), big.NewInt(1))

	require.Equal(t, uint32(1), tree.IndexArray.Get(1).Index)
	require.Equal(t, uint32(0), tree.IndexArray.Get(1).NextIndex)
	require.Equal(t, maxVal, tree.IndexArray.Get(1).Value)
}

func TestIndexedMerkleTreeAppend(t *testing.T) {
	/*
		Reference (rust) implementation outpu
		indexed mt inited root [33, 133, 56, 184, 142, 166, 110, 161, 4, 140, 169, 247, 115, 33, 15, 181, 76, 89, 48, 126, 58, 86, 204, 81, 16, 121, 185, 77, 75, 152, 43, 15]
		non inclusion proof init NonInclusionProof { root: [33, 133, 56, 184, 142, 166, 110, 161, 4, 140, 169, 247, 115, 33, 15, 181, 76, 89, 48, 126, 58, 86, 204, 81, 16, 121, 185, 77, 75, 152, 43, 15], value: [0, 171, 159, 63, 33, 62, 94, 156, 27, 61, 216, 203, 164, 91, 229, 110, 16, 230, 124, 129, 133, 222, 159, 99, 235, 50, 181, 94, 203, 105, 23, 82], leaf_lower_range_value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], leaf_higher_range_value: [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255], leaf_index: 0, next_index: 1, merkle_proof: [[30, 164, 22, 238, 180, 2, 24, 181, 64, 193, 207, 184, 219, 233, 31, 109, 84, 232, 162, 158, 220, 48, 163, 158, 50, 107, 64, 87, 167, 217, 99, 245], [32, 152, 245, 251, 158, 35, 158, 171, 60, 234, 195, 242, 123, 129, 228, 129, 220, 49, 36, 213, 95, 254, 213, 35, 168, 57, 238, 132, 70, 182, 72, 100], [16, 105, 103, 61, 205, 177, 34, 99, 223, 48, 26, 111, 245, 132, 167, 236, 38, 26, 68, 203, 157, 198, 141, 240, 103, 164, 119, 68, 96, 177, 241, 225], [24, 244, 51, 49, 83, 126, 226, 175, 46, 61, 117, 141, 80, 247, 33, 6, 70, 124, 110, 234, 80, 55, 29, 213, 40, 213, 126, 178, 184, 86, 210, 56], [7, 249, 216, 55, 203, 23, 176, 211, 99, 32, 255, 233, 59, 165, 35, 69, 241, 183, 40, 87, 26, 86, 130, 101, 202, 172, 151, 85, 157, 188, 149, 42], [43, 148, 207, 94, 135, 70, 179, 245, 201, 99, 31, 76, 93, 243, 41, 7, 166, 153, 197, 140, 148, 178, 173, 77, 123, 92, 236, 22, 57, 24, 63, 85], [45, 238, 147, 197, 166, 102, 69, 150, 70, 234, 125, 34, 204, 169, 225, 188, 254, 215, 30, 105, 81, 185, 83, 97, 29, 17, 221, 163, 46, 160, 157, 120], [7, 130, 149, 229, 162, 43, 132, 233, 130, 207, 96, 30, 182, 57, 89, 123, 139, 5, 21, 168, 140, 181, 172, 127, 168, 164, 170, 190, 60, 135, 52, 157], [47, 165, 229, 241, 143, 96, 39, 166, 80, 27, 236, 134, 69, 100, 71, 42, 97, 107, 46, 39, 74, 65, 33, 26, 68, 76, 190, 58, 153, 243, 204, 97], [14, 136, 67, 118, 208, 216, 253, 33, 236, 183, 128, 56, 158, 148, 31, 102, 228, 94, 122, 204, 227, 226, 40, 171, 62, 33, 86, 166, 20, 252, 215, 71], [27, 114, 1, 218, 114, 73, 79, 30, 40, 113, 122, 209, 165, 46, 180, 105, 249, 88, 146, 249, 87, 113, 53, 51, 222, 97, 117, 229, 218, 25, 10, 242], [31, 141, 136, 34, 114, 94, 54, 56, 82, 0, 192, 178, 1, 36, 152, 25, 166, 230, 225, 228, 101, 8, 8, 181, 190, 188, 107, 250, 206, 125, 118, 54], [44, 93, 130, 246, 108, 145, 75, 175, 185, 112, 21, 137, 186, 140, 252, 251, 97, 98, 176, 161, 42, 207, 136, 168, 208, 135, 154, 4, 113, 181, 248, 90], [20, 197, 65, 72, 160, 148, 11, 184, 32, 149, 127, 90, 223, 63, 161, 19, 78, 245, 196, 170, 161, 19, 244, 100, 100, 88, 242, 112, 224, 191, 191, 208], [25, 13, 51, 177, 47, 152, 111, 150, 30, 16, 192, 238, 68, 216, 185, 175, 17, 190, 37, 88, 140, 173, 137, 212, 22, 17, 142, 75, 244, 235, 232, 12], [34, 249, 138, 169, 206, 112, 65, 82, 172, 23, 53, 73, 20, 173, 115, 237, 17, 103, 174, 101, 150, 175, 81, 10, 165, 179, 100, 147, 37, 224, 108, 146], [42, 124, 124, 155, 108, 229, 136, 11, 159, 111, 34, 141, 114, 191, 106, 87, 90, 82, 111, 41, 198, 110, 204, 238, 248, 183, 83, 211, 139, 186, 115, 35], [46, 129, 134, 229, 88, 105, 142, 193, 198, 122, 249, 193, 77, 70, 63, 252, 71, 0, 67, 201, 194, 152, 139, 149, 77, 117, 221, 100, 63, 54, 185, 146], [15, 87, 197, 87, 30, 154, 78, 171, 73, 226, 200, 207, 5, 13, 174, 148, 138, 239, 110, 173, 100, 115, 146, 39, 53, 70, 36, 157, 28, 31, 241, 15], [24, 48, 238, 103, 181, 251, 85, 74, 213, 246, 61, 67, 136, 128, 14, 28, 254, 120, 227, 16, 105, 125, 70, 228, 60, 156, 227, 97, 52, 247, 44, 202], [33, 52, 231, 106, 197, 210, 26, 171, 24, 108, 43, 225, 221, 143, 132, 238, 136, 10, 30, 70, 234, 247, 18, 249, 211, 113, 182, 223, 34, 25, 31, 62], [25, 223, 144, 236, 132, 78, 188, 79, 254, 235, 216, 102, 243, 56, 89, 176, 192, 81, 216, 201, 88, 238, 58, 168, 143, 143, 141, 243, 219, 145, 165, 177], [24, 204, 162, 166, 107, 92, 7, 135, 152, 30, 105, 174, 253, 132, 133, 45, 116, 175, 14, 147, 239, 73, 18, 180, 100, 140, 5, 247, 34, 239, 229, 43], [35, 136, 144, 148, 21, 35, 13, 27, 77, 19, 4, 210, 213, 79, 71, 58, 98, 131, 56, 242, 239, 173, 131, 250, 223, 5, 100, 69, 73, 210, 83, 141], [39, 23, 31, 180, 169, 123, 108, 192, 233, 232, 245, 67, 181, 41, 77, 232, 102, 162, 175, 44, 156, 141, 11, 29, 150, 230, 115, 228, 82, 158, 213, 64], [47, 246, 101, 5, 64, 246, 41, 253, 87, 17, 160, 188, 116, 252, 13, 40, 220, 178, 48, 185, 57, 37, 131, 229, 248, 213, 150, 150, 221, 230, 174, 33]] }
		indexed mt with one append [31, 159, 196, 171, 68, 16, 213, 28, 158, 200, 223, 91, 244, 193, 188, 162, 50, 68, 54, 244, 116, 44, 153, 65, 209, 9, 47, 98, 126, 89, 131, 158]
		indexed array state element 0 IndexedElement { index: 0, value: 0, next_index: 2 }
		indexed array state element 1 IndexedElement { index: 1, value: 452312848583266388373324160190187140051835877600158453279131187530910662655, next_index: 0 }
		indexed array state element 2 IndexedElement { index: 2, value: 30, next_index: 1 }
		non inclusion proof address 2 NonInclusionProof { root: [31, 159, 196, 171, 68, 16, 213, 28, 158, 200, 223, 91, 244, 193, 188, 162, 50, 68, 54, 244, 116, 44, 153, 65, 209, 9, 47, 98, 126, 89, 131, 158], value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42], leaf_lower_range_value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30], leaf_higher_range_value: [0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255], leaf_index: 2, next_index: 1, merkle_proof: [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [18, 229, 201, 44, 165, 118, 84, 222, 209, 217, 57, 52, 169, 53, 5, 206, 20, 174, 62, 214, 23, 199, 249, 52, 103, 60, 29, 56, 48, 151, 87, 96], [16, 105, 103, 61, 205, 177, 34, 99, 223, 48, 26, 111, 245, 132, 167, 236, 38, 26, 68, 203, 157, 198, 141, 240, 103, 164, 119, 68, 96, 177, 241, 225], [24, 244, 51, 49, 83, 126, 226, 175, 46, 61, 117, 141, 80, 247, 33, 6, 70, 124, 110, 234, 80, 55, 29, 213, 40, 213, 126, 178, 184, 86, 210, 56], [7, 249, 216, 55, 203, 23, 176, 211, 99, 32, 255, 233, 59, 165, 35, 69, 241, 183, 40, 87, 26, 86, 130, 101, 202, 172, 151, 85, 157, 188, 149, 42], [43, 148, 207, 94, 135, 70, 179, 245, 201, 99, 31, 76, 93, 243, 41, 7, 166, 153, 197, 140, 148, 178, 173, 77, 123, 92, 236, 22, 57, 24, 63, 85], [45, 238, 147, 197, 166, 102, 69, 150, 70, 234, 125, 34, 204, 169, 225, 188, 254, 215, 30, 105, 81, 185, 83, 97, 29, 17, 221, 163, 46, 160, 157, 120], [7, 130, 149, 229, 162, 43, 132, 233, 130, 207, 96, 30, 182, 57, 89, 123, 139, 5, 21, 168, 140, 181, 172, 127, 168, 164, 170, 190, 60, 135, 52, 157], [47, 165, 229, 241, 143, 96, 39, 166, 80, 27, 236, 134, 69, 100, 71, 42, 97, 107, 46, 39, 74, 65, 33, 26, 68, 76, 190, 58, 153, 243, 204, 97], [14, 136, 67, 118, 208, 216, 253, 33, 236, 183, 128, 56, 158, 148, 31, 102, 228, 94, 122, 204, 227, 226, 40, 171, 62, 33, 86, 166, 20, 252, 215, 71], [27, 114, 1, 218, 114, 73, 79, 30, 40, 113, 122, 209, 165, 46, 180, 105, 249, 88, 146, 249, 87, 113, 53, 51, 222, 97, 117, 229, 218, 25, 10, 242], [31, 141, 136, 34, 114, 94, 54, 56, 82, 0, 192, 178, 1, 36, 152, 25, 166, 230, 225, 228, 101, 8, 8, 181, 190, 188, 107, 250, 206, 125, 118, 54], [44, 93, 130, 246, 108, 145, 75, 175, 185, 112, 21, 137, 186, 140, 252, 251, 97, 98, 176, 161, 42, 207, 136, 168, 208, 135, 154, 4, 113, 181, 248, 90], [20, 197, 65, 72, 160, 148, 11, 184, 32, 149, 127, 90, 223, 63, 161, 19, 78, 245, 196, 170, 161, 19, 244, 100, 100, 88, 242, 112, 224, 191, 191, 208], [25, 13, 51, 177, 47, 152, 111, 150, 30, 16, 192, 238, 68, 216, 185, 175, 17, 190, 37, 88, 140, 173, 137, 212, 22, 17, 142, 75, 244, 235, 232, 12], [34, 249, 138, 169, 206, 112, 65, 82, 172, 23, 53, 73, 20, 173, 115, 237, 17, 103, 174, 101, 150, 175, 81, 10, 165, 179, 100, 147, 37, 224, 108, 146], [42, 124, 124, 155, 108, 229, 136, 11, 159, 111, 34, 141, 114, 191, 106, 87, 90, 82, 111, 41, 198, 110, 204, 238, 248, 183, 83, 211, 139, 186, 115, 35], [46, 129, 134, 229, 88, 105, 142, 193, 198, 122, 249, 193, 77, 70, 63, 252, 71, 0, 67, 201, 194, 152, 139, 149, 77, 117, 221, 100, 63, 54, 185, 146], [15, 87, 197, 87, 30, 154, 78, 171, 73, 226, 200, 207, 5, 13, 174, 148, 138, 239, 110, 173, 100, 115, 146, 39, 53, 70, 36, 157, 28, 31, 241, 15], [24, 48, 238, 103, 181, 251, 85, 74, 213, 246, 61, 67, 136, 128, 14, 28, 254, 120, 227, 16, 105, 125, 70, 228, 60, 156, 227, 97, 52, 247, 44, 202], [33, 52, 231, 106, 197, 210, 26, 171, 24, 108, 43, 225, 221, 143, 132, 238, 136, 10, 30, 70, 234, 247, 18, 249, 211, 113, 182, 223, 34, 25, 31, 62], [25, 223, 144, 236, 132, 78, 188, 79, 254, 235, 216, 102, 243, 56, 89, 176, 192, 81, 216, 201, 88, 238, 58, 168, 143, 143, 141, 243, 219, 145, 165, 177], [24, 204, 162, 166, 107, 92, 7, 135, 152, 30, 105, 174, 253, 132, 133, 45, 116, 175, 14, 147, 239, 73, 18, 180, 100, 140, 5, 247, 34, 239, 229, 43], [35, 136, 144, 148, 21, 35, 13, 27, 77, 19, 4, 210, 213, 79, 71, 58, 98, 131, 56, 242, 239, 173, 131, 250, 223, 5, 100, 69, 73, 210, 83, 141], [39, 23, 31, 180, 169, 123, 108, 192, 233, 232, 245, 67, 181, 41, 77, 232, 102, 162, 175, 44, 156, 141, 11, 29, 150, 230, 115, 228, 82, 158, 213, 64], [47, 246, 101, 5, 64, 246, 41, 253, 87, 17, 160, 188, 116, 252, 13, 40, 220, 178, 48, 185, 57, 37, 131, 229, 248, 213, 150, 150, 221, 230, 174, 33]] }
		indexed mt with two appends [1, 185, 99, 233, 59, 202, 51, 222, 224, 31, 119, 180, 76, 104, 72, 27, 152, 12, 236, 78, 81, 60, 87, 158, 237, 1, 176, 9, 155, 166, 108, 89]
		indexed array state element 0 IndexedElement { index: 0, value: 0, next_index: 2 }
		indexed array state element 1 IndexedElement { index: 1, value: 452312848583266388373324160190187140051835877600158453279131187530910662655, next_index: 0 }
		indexed array state element 2 IndexedElement { index: 2, value: 30, next_index: 3 }
		indexed array state element 3 IndexedElement { index: 3, value: 42, next_index: 1 }
		indexed mt with three appends [41, 143, 181, 2, 66, 117, 37, 226, 134, 212, 45, 95, 114, 60, 189, 18, 44, 155, 132, 148, 41, 54, 131, 106, 61, 120, 237, 168, 118, 198, 63, 116]
		non inclusion proof address 3 NonInclusionProof { root: [1, 185, 99, 233, 59, 202, 51, 222, 224, 31, 119, 180, 76, 104, 72, 27, 152, 12, 236, 78, 81, 60, 87, 158, 237, 1, 176, 9, 155, 166, 108, 89], value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12], leaf_lower_range_value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], leaf_higher_range_value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30], leaf_index: 0, next_index: 2, merkle_proof: [[30, 164, 22, 238, 180, 2, 24, 181, 64, 193, 207, 184, 219, 233, 31, 109, 84, 232, 162, 158, 220, 48, 163, 158, 50, 107, 64, 87, 167, 217, 99, 245], [43, 152, 0, 169, 196, 194, 43, 216, 106, 218, 53, 230, 207, 92, 177, 234, 56, 87, 194, 42, 31, 53, 145, 250, 212, 51, 229, 176, 218, 21, 194, 77], [16, 105, 103, 61, 205, 177, 34, 99, 223, 48, 26, 111, 245, 132, 167, 236, 38, 26, 68, 203, 157, 198, 141, 240, 103, 164, 119, 68, 96, 177, 241, 225], [24, 244, 51, 49, 83, 126, 226, 175, 46, 61, 117, 141, 80, 247, 33, 6, 70, 124, 110, 234, 80, 55, 29, 213, 40, 213, 126, 178, 184, 86, 210, 56], [7, 249, 216, 55, 203, 23, 176, 211, 99, 32, 255, 233, 59, 165, 35, 69, 241, 183, 40, 87, 26, 86, 130, 101, 202, 172, 151, 85, 157, 188, 149, 42], [43, 148, 207, 94, 135, 70, 179, 245, 201, 99, 31, 76, 93, 243, 41, 7, 166, 153, 197, 140, 148, 178, 173, 77, 123, 92, 236, 22, 57, 24, 63, 85], [45, 238, 147, 197, 166, 102, 69, 150, 70, 234, 125, 34, 204, 169, 225, 188, 254, 215, 30, 105, 81, 185, 83, 97, 29, 17, 221, 163, 46, 160, 157, 120], [7, 130, 149, 229, 162, 43, 132, 233, 130, 207, 96, 30, 182, 57, 89, 123, 139, 5, 21, 168, 140, 181, 172, 127, 168, 164, 170, 190, 60, 135, 52, 157], [47, 165, 229, 241, 143, 96, 39, 166, 80, 27, 236, 134, 69, 100, 71, 42, 97, 107, 46, 39, 74, 65, 33, 26, 68, 76, 190, 58, 153, 243, 204, 97], [14, 136, 67, 118, 208, 216, 253, 33, 236, 183, 128, 56, 158, 148, 31, 102, 228, 94, 122, 204, 227, 226, 40, 171, 62, 33, 86, 166, 20, 252, 215, 71], [27, 114, 1, 218, 114, 73, 79, 30, 40, 113, 122, 209, 165, 46, 180, 105, 249, 88, 146, 249, 87, 113, 53, 51, 222, 97, 117, 229, 218, 25, 10, 242], [31, 141, 136, 34, 114, 94, 54, 56, 82, 0, 192, 178, 1, 36, 152, 25, 166, 230, 225, 228, 101, 8, 8, 181, 190, 188, 107, 250, 206, 125, 118, 54], [44, 93, 130, 246, 108, 145, 75, 175, 185, 112, 21, 137, 186, 140, 252, 251, 97, 98, 176, 161, 42, 207, 136, 168, 208, 135, 154, 4, 113, 181, 248, 90], [20, 197, 65, 72, 160, 148, 11, 184, 32, 149, 127, 90, 223, 63, 161, 19, 78, 245, 196, 170, 161, 19, 244, 100, 100, 88, 242, 112, 224, 191, 191, 208], [25, 13, 51, 177, 47, 152, 111, 150, 30, 16, 192, 238, 68, 216, 185, 175, 17, 190, 37, 88, 140, 173, 137, 212, 22, 17, 142, 75, 244, 235, 232, 12], [34, 249, 138, 169, 206, 112, 65, 82, 172, 23, 53, 73, 20, 173, 115, 237, 17, 103, 174, 101, 150, 175, 81, 10, 165, 179, 100, 147, 37, 224, 108, 146], [42, 124, 124, 155, 108, 229, 136, 11, 159, 111, 34, 141, 114, 191, 106, 87, 90, 82, 111, 41, 198, 110, 204, 238, 248, 183, 83, 211, 139, 186, 115, 35], [46, 129, 134, 229, 88, 105, 142, 193, 198, 122, 249, 193, 77, 70, 63, 252, 71, 0, 67, 201, 194, 152, 139, 149, 77, 117, 221, 100, 63, 54, 185, 146], [15, 87, 197, 87, 30, 154, 78, 171, 73, 226, 200, 207, 5, 13, 174, 148, 138, 239, 110, 173, 100, 115, 146, 39, 53, 70, 36, 157, 28, 31, 241, 15], [24, 48, 238, 103, 181, 251, 85, 74, 213, 246, 61, 67, 136, 128, 14, 28, 254, 120, 227, 16, 105, 125, 70, 228, 60, 156, 227, 97, 52, 247, 44, 202], [33, 52, 231, 106, 197, 210, 26, 171, 24, 108, 43, 225, 221, 143, 132, 238, 136, 10, 30, 70, 234, 247, 18, 249, 211, 113, 182, 223, 34, 25, 31, 62], [25, 223, 144, 236, 132, 78, 188, 79, 254, 235, 216, 102, 243, 56, 89, 176, 192, 81, 216, 201, 88, 238, 58, 168, 143, 143, 141, 243, 219, 145, 165, 177], [24, 204, 162, 166, 107, 92, 7, 135, 152, 30, 105, 174, 253, 132, 133, 45, 116, 175, 14, 147, 239, 73, 18, 180, 100, 140, 5, 247, 34, 239, 229, 43], [35, 136, 144, 148, 21, 35, 13, 27, 77, 19, 4, 210, 213, 79, 71, 58, 98, 131, 56, 242, 239, 173, 131, 250, 223, 5, 100, 69, 73, 210, 83, 141], [39, 23, 31, 180, 169, 123, 108, 192, 233, 232, 245, 67, 181, 41, 77, 232, 102, 162, 175, 44, 156, 141, 11, 29, 150, 230, 115, 228, 82, 158, 213, 64], [47, 246, 101, 5, 64, 246, 41, 253, 87, 17, 160, 188, 116, 252, 13, 40, 220, 178, 48, 185, 57, 37, 131, 229, 248, 213, 150, 150, 221, 230, 174, 33]] }
		indexed array state element 0 IndexedElement { index: 0, value: 0, next_index: 4 }
		indexed array state element 1 IndexedElement { index: 1, value: 452312848583266388373324160190187140051835877600158453279131187530910662655, next_index: 0 }
		indexed array state element 2 IndexedElement { index: 2, value: 30, next_index: 3 }
		indexed array state element 3 IndexedElement { index: 3, value: 42, next_index: 1 }
		indexed array state element 4 IndexedElement { index: 4, value: 12, next_index: 2 }
	*/
	tree, err := NewIndexedMerkleTree(26)
	require.NoError(t, err)

	err = tree.Init()
	require.NoError(t, err)

	value := big.NewInt(30)
	err = tree.Append(value)
	require.NoError(t, err)

	expectedRootFirstAppend := []byte{31, 159, 196, 171, 68, 16, 213, 28, 158, 200, 223, 91, 244, 193, 188, 162, 50, 68, 54, 244, 116, 44, 153, 65, 209, 9, 47, 98, 126, 89, 131, 158}

	root := tree.Tree.Root.Bytes()
	fmt.Println("indexed mt with one append", root)

	require.Equal(t, expectedRootFirstAppend, root)

	require.Equal(t, uint32(0), tree.IndexArray.Get(0).Index)
	require.Equal(t, uint32(2), tree.IndexArray.Get(0).NextIndex)
	require.Equal(t, "0", tree.IndexArray.Get(0).Value.String())

	maxVal := new(big.Int).Sub(new(big.Int).Lsh(big.NewInt(1), 248), big.NewInt(1))

	require.Equal(t, uint32(1), tree.IndexArray.Get(1).Index)
	require.Equal(t, uint32(0), tree.IndexArray.Get(1).NextIndex)
	require.Equal(t, maxVal, tree.IndexArray.Get(1).Value)

	require.Equal(t, uint32(2), tree.IndexArray.Get(2).Index)
	require.Equal(t, uint32(1), tree.IndexArray.Get(2).NextIndex)
	require.Equal(t, "30", tree.IndexArray.Get(2).Value.String())

	value = big.NewInt(42)
	err = tree.Append(value)
	require.NoError(t, err)

	expectedRootSecondAppend := []byte{1, 185, 99, 233, 59, 202, 51, 222, 224, 31, 119, 180, 76, 104, 72, 27, 152, 12, 236, 78, 81, 60, 87, 158, 237, 1, 176, 9, 155, 166, 108, 89}

	root = tree.Tree.Root.Bytes()

	fmt.Println("indexed mt with two appends", root)
	require.Equal(t, expectedRootSecondAppend, root)

	value = big.NewInt(12)
	err = tree.Append(value)
	require.NoError(t, err)

	expectedRootThirdAttempt := []byte{41, 143, 181, 2, 66, 117, 37, 226, 134, 212, 45, 95, 114, 60, 189, 18, 44, 155, 132, 148, 41, 54, 131, 106, 61, 120, 237, 168, 118, 198, 63, 116}

	root = tree.Tree.Root.Bytes()
	fmt.Println("indexed mt with three appends", root)
	require.Equal(t, expectedRootThirdAttempt, root)
}