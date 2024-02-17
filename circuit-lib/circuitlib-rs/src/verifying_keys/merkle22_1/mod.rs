use groth16_solana::groth16::Groth16Verifyingkey;

pub const VERIFYINGKEY: Groth16Verifyingkey = Groth16Verifyingkey {
    nr_pubinputs: 3,

    vk_alpha_g1: [
        45, 77, 154, 167, 227, 2, 217, 223, 65, 116, 157, 85, 7, 148, 157, 5, 219, 234, 51, 251,
        177, 108, 100, 59, 34, 245, 153, 162, 190, 109, 242, 226, 20, 190, 221, 80, 60, 55, 206,
        176, 97, 216, 236, 96, 32, 159, 227, 69, 206, 137, 131, 10, 25, 35, 3, 1, 240, 118, 202,
        255, 0, 77, 25, 38,
    ],

    vk_beta_g2: [
        9, 103, 3, 47, 203, 247, 118, 209, 175, 201, 133, 248, 136, 119, 241, 130, 211, 132, 128,
        166, 83, 242, 222, 202, 169, 121, 76, 188, 59, 243, 6, 12, 14, 24, 120, 71, 173, 76, 121,
        131, 116, 208, 214, 115, 43, 245, 1, 132, 125, 214, 139, 192, 224, 113, 36, 30, 2, 19, 188,
        127, 193, 61, 183, 171, 48, 76, 251, 209, 224, 138, 112, 74, 153, 245, 232, 71, 217, 63,
        140, 60, 170, 253, 222, 196, 107, 122, 13, 55, 157, 166, 154, 77, 17, 35, 70, 167, 23, 57,
        193, 177, 164, 87, 168, 199, 49, 49, 35, 210, 77, 47, 145, 146, 248, 150, 183, 198, 62,
        234, 5, 169, 213, 127, 6, 84, 122, 208, 206, 200,
    ],

    vk_gamme_g2: [
        25, 142, 147, 147, 146, 13, 72, 58, 114, 96, 191, 183, 49, 251, 93, 37, 241, 170, 73, 51,
        53, 169, 231, 18, 151, 228, 133, 183, 174, 243, 18, 194, 24, 0, 222, 239, 18, 31, 30, 118,
        66, 106, 0, 102, 94, 92, 68, 121, 103, 67, 34, 212, 247, 94, 218, 221, 70, 222, 189, 92,
        217, 146, 246, 237, 9, 6, 137, 208, 88, 95, 240, 117, 236, 158, 153, 173, 105, 12, 51, 149,
        188, 75, 49, 51, 112, 179, 142, 243, 85, 172, 218, 220, 209, 34, 151, 91, 18, 200, 94, 165,
        219, 140, 109, 235, 74, 171, 113, 128, 141, 203, 64, 143, 227, 209, 231, 105, 12, 67, 211,
        123, 76, 230, 204, 1, 102, 250, 125, 170,
    ],

    vk_delta_g2: [
        7, 178, 5, 206, 215, 69, 47, 25, 64, 34, 185, 66, 175, 11, 80, 9, 225, 212, 47, 90, 142,
        191, 216, 211, 33, 58, 156, 232, 42, 171, 170, 82, 18, 56, 96, 193, 159, 168, 122, 123,
        130, 85, 135, 61, 13, 136, 245, 66, 74, 34, 85, 32, 5, 13, 246, 33, 56, 120, 176, 68, 7, 3,
        26, 175, 37, 254, 255, 153, 24, 37, 126, 229, 159, 89, 133, 193, 25, 69, 114, 219, 14, 96,
        127, 53, 23, 91, 190, 33, 241, 224, 249, 228, 149, 53, 17, 16, 18, 145, 132, 181, 175, 72,
        77, 168, 108, 91, 229, 163, 244, 23, 100, 27, 84, 224, 51, 161, 60, 73, 156, 96, 177, 62,
        70, 10, 36, 73, 125, 32,
    ],

    vk_ic: &[
        [
            7, 231, 26, 77, 62, 96, 43, 56, 199, 247, 253, 4, 23, 65, 113, 18, 228, 233, 114, 193,
            204, 122, 104, 188, 164, 195, 67, 74, 243, 108, 4, 66, 39, 154, 29, 72, 223, 177, 58,
            25, 195, 203, 60, 34, 253, 84, 133, 9, 162, 27, 65, 39, 236, 180, 142, 229, 195, 70,
            24, 45, 245, 81, 154, 31,
        ],
        [
            39, 10, 194, 229, 38, 99, 158, 58, 242, 51, 110, 219, 52, 199, 220, 217, 37, 110, 128,
            230, 15, 49, 142, 73, 79, 237, 32, 61, 118, 96, 199, 236, 25, 230, 170, 122, 87, 41,
            81, 216, 135, 165, 158, 214, 33, 105, 145, 163, 157, 158, 249, 160, 141, 162, 29, 6,
            39, 135, 170, 201, 101, 116, 22, 208,
        ],
        [
            23, 161, 215, 152, 203, 204, 144, 206, 74, 128, 173, 154, 199, 225, 175, 167, 92, 224,
            2, 64, 78, 135, 82, 194, 57, 141, 54, 82, 39, 62, 224, 111, 23, 173, 200, 19, 27, 129,
            149, 44, 219, 18, 216, 69, 180, 248, 88, 93, 21, 75, 55, 89, 156, 135, 217, 23, 82,
            131, 115, 130, 236, 251, 47, 74,
        ],
    ],
};
