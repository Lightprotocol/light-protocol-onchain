use anchor_lang::prelude::*;
use groth16_solana::groth16::Groth16Verifyingkey;

pub const VERIFYINGKEY_TRANSACTION_MASP10_MAIN: Groth16Verifyingkey =
    Groth16Verifyingkey {
        nr_pubinputs: 17,
        vk_alpha_g1: [
            45, 77, 154, 167, 227, 2, 217, 223, 65, 116, 157, 85, 7, 148, 157, 5, 219, 234, 51,
            251, 177, 108, 100, 59, 34, 245, 153, 162, 190, 109, 242, 226, 20, 190, 221, 80, 60,
            55, 206, 176, 97, 216, 236, 96, 32, 159, 227, 69, 206, 137, 131, 10, 25, 35, 3, 1, 240,
            118, 202, 255, 0, 77, 25, 38,
        ],

        vk_beta_g2: [
            9, 103, 3, 47, 203, 247, 118, 209, 175, 201, 133, 248, 136, 119, 241, 130, 211, 132,
            128, 166, 83, 242, 222, 202, 169, 121, 76, 188, 59, 243, 6, 12, 14, 24, 120, 71, 173,
            76, 121, 131, 116, 208, 214, 115, 43, 245, 1, 132, 125, 214, 139, 192, 224, 113, 36,
            30, 2, 19, 188, 127, 193, 61, 183, 171, 48, 76, 251, 209, 224, 138, 112, 74, 153, 245,
            232, 71, 217, 63, 140, 60, 170, 253, 222, 196, 107, 122, 13, 55, 157, 166, 154, 77, 17,
            35, 70, 167, 23, 57, 193, 177, 164, 87, 168, 199, 49, 49, 35, 210, 77, 47, 145, 146,
            248, 150, 183, 198, 62, 234, 5, 169, 213, 127, 6, 84, 122, 208, 206, 200,
        ],

        vk_gamme_g2: [
            25, 142, 147, 147, 146, 13, 72, 58, 114, 96, 191, 183, 49, 251, 93, 37, 241, 170, 73,
            51, 53, 169, 231, 18, 151, 228, 133, 183, 174, 243, 18, 194, 24, 0, 222, 239, 18, 31,
            30, 118, 66, 106, 0, 102, 94, 92, 68, 121, 103, 67, 34, 212, 247, 94, 218, 221, 70,
            222, 189, 92, 217, 146, 246, 237, 9, 6, 137, 208, 88, 95, 240, 117, 236, 158, 153, 173,
            105, 12, 51, 149, 188, 75, 49, 51, 112, 179, 142, 243, 85, 172, 218, 220, 209, 34, 151,
            91, 18, 200, 94, 165, 219, 140, 109, 235, 74, 171, 113, 128, 141, 203, 64, 143, 227,
            209, 231, 105, 12, 67, 211, 123, 76, 230, 204, 1, 102, 250, 125, 170,
        ],

        vk_delta_g2: [
            42, 98, 188, 78, 130, 117, 125, 55, 17, 74, 174, 242, 90, 223, 234, 118, 2, 26, 131,
            35, 243, 224, 251, 131, 66, 23, 193, 171, 108, 255, 211, 250, 48, 63, 212, 62, 207, 90,
            6, 210, 48, 131, 192, 24, 165, 137, 203, 64, 51, 119, 250, 221, 103, 66, 84, 240, 171,
            168, 147, 160, 17, 220, 98, 26, 10, 214, 247, 221, 229, 245, 171, 16, 107, 54, 217, 97,
            11, 169, 60, 239, 157, 178, 148, 238, 92, 114, 252, 222, 71, 91, 89, 151, 87, 157, 195,
            217, 39, 179, 70, 14, 148, 255, 20, 28, 35, 7, 152, 4, 75, 34, 174, 102, 209, 204, 50,
            226, 130, 204, 134, 163, 122, 107, 106, 74, 13, 56, 252, 218,
        ],

        vk_ic: &[
            [
                22, 136, 127, 232, 17, 135, 19, 187, 56, 49, 254, 177, 244, 32, 125, 230, 47, 180,
                102, 41, 111, 6, 160, 248, 218, 198, 129, 219, 10, 138, 80, 114, 1, 253, 105, 191,
                180, 37, 16, 99, 171, 105, 89, 158, 231, 25, 194, 22, 45, 33, 122, 171, 62, 67,
                176, 72, 45, 161, 58, 162, 132, 161, 120, 62,
            ],
            [
                7, 56, 166, 15, 187, 232, 229, 222, 5, 51, 52, 165, 162, 91, 21, 206, 176, 107,
                207, 182, 241, 226, 173, 198, 100, 28, 61, 41, 10, 43, 23, 125, 43, 151, 140, 238,
                91, 2, 10, 30, 93, 22, 179, 216, 0, 32, 141, 48, 186, 117, 79, 214, 54, 156, 182,
                62, 41, 217, 239, 5, 154, 98, 198, 62,
            ],
            [
                24, 169, 88, 180, 77, 107, 75, 79, 171, 10, 3, 198, 251, 211, 156, 88, 166, 196,
                93, 53, 69, 60, 116, 14, 225, 73, 201, 83, 111, 109, 62, 134, 28, 60, 227, 12, 63,
                132, 5, 48, 13, 215, 204, 106, 18, 36, 138, 104, 92, 60, 219, 140, 206, 50, 254,
                134, 55, 2, 168, 101, 166, 51, 193, 136,
            ],
            [
                29, 111, 246, 168, 44, 208, 99, 128, 157, 153, 90, 173, 160, 42, 37, 253, 174, 128,
                245, 150, 247, 244, 179, 53, 37, 197, 253, 155, 206, 199, 56, 18, 40, 54, 77, 10,
                156, 172, 57, 143, 231, 51, 8, 201, 76, 205, 229, 13, 56, 122, 196, 123, 201, 88,
                116, 220, 212, 123, 121, 175, 164, 3, 123, 76,
            ],
            [
                21, 140, 231, 175, 205, 143, 243, 27, 225, 103, 150, 36, 221, 240, 104, 225, 25, 6,
                254, 182, 25, 64, 90, 13, 223, 176, 71, 192, 106, 208, 15, 73, 34, 64, 233, 57, 4,
                29, 165, 173, 45, 68, 46, 26, 9, 115, 111, 55, 153, 210, 175, 24, 173, 110, 227,
                206, 70, 2, 223, 66, 20, 13, 232, 230,
            ],
            [
                23, 38, 31, 49, 220, 182, 214, 179, 85, 55, 241, 151, 4, 57, 193, 97, 76, 35, 18,
                249, 49, 250, 104, 207, 42, 214, 118, 172, 62, 201, 40, 183, 16, 180, 41, 226, 9,
                45, 7, 52, 72, 66, 61, 86, 90, 115, 211, 1, 241, 12, 233, 19, 240, 73, 155, 85,
                215, 93, 219, 159, 82, 46, 68, 156,
            ],
            [
                25, 82, 60, 227, 65, 20, 22, 199, 116, 119, 252, 111, 72, 44, 218, 169, 55, 136,
                170, 25, 11, 159, 22, 109, 122, 214, 130, 141, 240, 63, 179, 116, 12, 123, 124,
                210, 27, 92, 132, 166, 37, 201, 203, 226, 67, 6, 233, 63, 49, 63, 99, 91, 154, 55,
                188, 27, 137, 247, 150, 118, 93, 156, 125, 241,
            ],
            [
                16, 61, 3, 143, 196, 80, 198, 5, 65, 152, 116, 108, 14, 99, 248, 58, 10, 136, 166,
                207, 176, 146, 54, 215, 75, 36, 186, 205, 248, 5, 234, 12, 45, 159, 146, 67, 204,
                185, 99, 201, 145, 198, 48, 228, 113, 151, 0, 195, 132, 184, 113, 86, 2, 1, 49,
                213, 60, 9, 21, 120, 204, 55, 100, 176,
            ],
            [
                5, 82, 25, 120, 7, 82, 39, 109, 7, 205, 237, 197, 41, 97, 93, 42, 216, 64, 233,
                241, 174, 151, 158, 112, 29, 84, 253, 217, 130, 217, 252, 248, 42, 33, 137, 157,
                250, 105, 235, 44, 248, 91, 207, 54, 250, 75, 35, 208, 109, 24, 107, 232, 117, 104,
                200, 251, 120, 76, 186, 187, 136, 81, 209, 23,
            ],
            [
                41, 102, 236, 147, 98, 15, 237, 238, 99, 200, 182, 226, 212, 172, 196, 161, 77,
                110, 115, 90, 220, 206, 139, 27, 81, 245, 117, 136, 40, 98, 243, 104, 45, 173, 118,
                106, 4, 0, 84, 40, 175, 71, 60, 38, 85, 204, 130, 33, 119, 218, 59, 184, 123, 242,
                200, 220, 54, 206, 1, 54, 120, 201, 148, 175,
            ],
            [
                23, 112, 79, 94, 143, 163, 186, 96, 23, 89, 100, 94, 78, 56, 141, 225, 8, 135, 22,
                129, 96, 29, 70, 204, 52, 43, 211, 123, 231, 255, 228, 102, 9, 178, 181, 109, 226,
                19, 97, 69, 158, 202, 250, 103, 66, 223, 221, 133, 183, 182, 248, 2, 12, 3, 177,
                232, 34, 150, 206, 138, 209, 14, 142, 142,
            ],
            [
                24, 208, 226, 127, 95, 126, 17, 28, 79, 136, 151, 167, 72, 102, 102, 79, 89, 127,
                251, 83, 46, 23, 125, 181, 191, 252, 215, 143, 14, 52, 10, 251, 17, 150, 242, 131,
                17, 178, 108, 77, 86, 9, 191, 166, 63, 152, 105, 109, 24, 4, 131, 159, 41, 148,
                249, 115, 185, 4, 195, 117, 225, 157, 21, 249,
            ],
            [
                48, 61, 41, 5, 60, 118, 214, 17, 71, 43, 200, 155, 30, 225, 41, 102, 90, 41, 122,
                67, 89, 124, 199, 168, 25, 40, 60, 232, 131, 188, 66, 112, 39, 202, 214, 253, 245,
                97, 175, 217, 184, 37, 6, 6, 58, 236, 105, 25, 236, 240, 104, 58, 63, 184, 227, 84,
                40, 199, 103, 199, 182, 133, 204, 74,
            ],
            [
                34, 100, 194, 226, 101, 148, 209, 29, 148, 90, 132, 194, 102, 64, 247, 4, 17, 82,
                165, 122, 121, 62, 234, 242, 103, 107, 170, 201, 225, 43, 28, 120, 0, 45, 206, 109,
                10, 111, 144, 104, 78, 85, 114, 200, 245, 17, 223, 25, 215, 128, 194, 207, 131,
                126, 156, 146, 232, 246, 127, 19, 119, 184, 32, 81,
            ],
            [
                24, 161, 154, 158, 46, 19, 110, 32, 164, 38, 10, 237, 69, 249, 196, 221, 7, 107,
                171, 177, 12, 134, 245, 113, 80, 82, 114, 84, 13, 203, 151, 98, 38, 220, 134, 221,
                44, 163, 59, 137, 121, 48, 37, 14, 31, 29, 189, 241, 140, 130, 122, 32, 241, 112,
                99, 43, 160, 158, 226, 237, 189, 38, 239, 114,
            ],
            [
                18, 80, 123, 227, 85, 186, 79, 159, 221, 161, 85, 171, 47, 167, 52, 202, 10, 170,
                203, 147, 148, 117, 53, 167, 208, 166, 59, 133, 69, 234, 55, 220, 45, 122, 254,
                149, 128, 147, 36, 142, 165, 209, 167, 137, 143, 205, 154, 144, 19, 179, 220, 182,
                52, 150, 197, 131, 237, 184, 89, 117, 10, 249, 128, 0,
            ],
            [
                19, 36, 30, 160, 129, 236, 80, 150, 241, 211, 39, 43, 171, 126, 158, 172, 185, 13,
                58, 139, 157, 216, 158, 206, 57, 60, 45, 157, 240, 189, 87, 29, 13, 26, 26, 39,
                163, 31, 23, 158, 85, 27, 50, 163, 49, 214, 49, 97, 96, 77, 20, 76, 54, 241, 50,
                180, 18, 9, 214, 251, 255, 122, 67, 59,
            ],
            [
                48, 24, 219, 250, 201, 196, 237, 84, 202, 201, 30, 34, 127, 90, 191, 94, 128, 19,
                174, 13, 110, 71, 128, 199, 200, 20, 112, 127, 101, 231, 106, 150, 34, 219, 72,
                244, 187, 30, 164, 111, 223, 240, 149, 85, 193, 89, 126, 142, 206, 121, 55, 246,
                224, 132, 8, 142, 112, 210, 244, 68, 94, 145, 75, 183,
            ],
        ],
    };
#[account]
pub struct ZKtransactionMasp10MainProofInputs {
    root: u8,
    public_amount_spl: u8,
    tx_integrity_hash: u8,
    public_amount_sol: u8,
    public_mint_pubkey: u8,
    input_nullifier: [u8; 10],
    output_commitment: [u8; 2],
    in_amount: [[u8; 2]; 10],
    in_private_key: [u8; 10],
    in_blinding: [u8; 10],
    in_app_data_hash: [u8; 10],
    in_path_indices: [u8; 10],
    in_path_elements: [[u8; 18]; 10],
    in_indices: [[[u8; 3]; 2]; 10],
    out_amount: [[u8; 2]; 2],
    out_pubkey: [u8; 2],
    out_blinding: [u8; 2],
    out_app_data_hash: [u8; 2],
    out_indices: [[[u8; 3]; 2]; 2],
    out_pool_type: [u8; 2],
    out_verifier_pubkey: [u8; 2],
    in_pool_type: [u8; 10],
    in_verifier_pubkey: [u8; 10],
    transaction_version: u8,
    asset_pubkeys: [u8; 3],
    internal_tx_integrity_hash: u8,
}
#[account]
pub struct ZKtransactionMasp10MainPublicInputs {
    root: u8,
    public_amount_spl: u8,
    tx_integrity_hash: u8,
    public_amount_sol: u8,
    public_mint_pubkey: u8,
    input_nullifier: [u8; 10],
    output_commitment: [u8; 2],
}
#[account]
pub struct InstructionDataLightInstructionTransactionMasp10MainSecond {
    root: [u8; 32],
    public_amount_spl: [u8; 32],
    tx_integrity_hash: [u8; 32],
    public_amount_sol: [u8; 32],
    public_mint_pubkey: [u8; 32],
    input_nullifier: [[u8; 32]; 10],
    output_commitment: [[u8; 32]; 2],
}
