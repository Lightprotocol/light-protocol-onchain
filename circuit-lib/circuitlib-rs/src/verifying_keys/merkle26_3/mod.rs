use groth16_solana::groth16::Groth16Verifyingkey;

pub const VERIFYINGKEY: Groth16Verifyingkey = Groth16Verifyingkey {
    nr_pubinputs: 7,

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
        42, 106, 136, 219, 184, 213, 167, 196, 143, 2, 1, 83, 168, 203, 171, 21, 142, 203, 99, 163,
        135, 76, 82, 199, 92, 241, 175, 104, 153, 211, 58, 11, 26, 32, 242, 185, 145, 122, 53, 32,
        36, 34, 234, 213, 151, 178, 23, 119, 230, 189, 46, 157, 159, 81, 233, 32, 34, 206, 190, 87,
        115, 179, 2, 236, 20, 20, 193, 12, 183, 21, 3, 159, 23, 112, 216, 159, 71, 18, 109, 255,
        237, 33, 147, 15, 90, 221, 239, 38, 39, 62, 106, 175, 161, 219, 79, 87, 8, 196, 202, 64,
        189, 2, 75, 110, 156, 217, 116, 114, 252, 28, 7, 170, 59, 22, 223, 60, 145, 57, 183, 247,
        153, 196, 206, 13, 54, 84, 239, 200,
    ],

    vk_ic: &[
        [
            5, 62, 86, 166, 105, 20, 244, 109, 2, 90, 230, 64, 165, 176, 58, 80, 83, 5, 96, 74,
            232, 163, 200, 85, 99, 89, 77, 38, 244, 149, 172, 125, 1, 96, 180, 170, 5, 18, 125, 88,
            106, 71, 236, 176, 154, 216, 141, 102, 205, 254, 176, 114, 71, 176, 149, 105, 9, 1,
            228, 104, 111, 37, 183, 55,
        ],
        [
            23, 175, 55, 66, 13, 186, 155, 100, 155, 55, 146, 192, 77, 240, 59, 3, 197, 145, 16,
            54, 52, 34, 39, 61, 216, 217, 145, 247, 104, 156, 41, 112, 22, 143, 210, 224, 35, 209,
            206, 172, 28, 102, 150, 109, 177, 152, 79, 231, 107, 75, 218, 220, 85, 107, 106, 90,
            79, 156, 88, 216, 242, 63, 219, 251,
        ],
        [
            28, 246, 184, 193, 216, 161, 197, 122, 30, 119, 26, 26, 98, 234, 161, 153, 56, 103, 77,
            10, 34, 201, 60, 191, 76, 149, 124, 45, 64, 42, 204, 113, 9, 87, 112, 208, 161, 30, 45,
            10, 126, 89, 231, 38, 198, 70, 189, 75, 159, 13, 203, 93, 91, 6, 91, 209, 66, 113, 217,
            72, 236, 147, 31, 227,
        ],
        [
            18, 79, 170, 220, 95, 254, 46, 43, 54, 104, 147, 210, 85, 187, 125, 163, 164, 216, 186,
            147, 231, 169, 88, 168, 189, 128, 138, 218, 169, 13, 88, 27, 36, 106, 169, 83, 6, 168,
            246, 151, 63, 91, 41, 230, 66, 41, 205, 196, 164, 64, 245, 209, 33, 41, 44, 31, 196,
            38, 39, 215, 36, 171, 52, 129,
        ],
        [
            18, 197, 64, 251, 97, 231, 161, 101, 225, 31, 193, 206, 212, 146, 248, 3, 116, 62, 98,
            41, 177, 227, 46, 55, 60, 6, 181, 4, 143, 80, 238, 230, 1, 185, 115, 29, 112, 26, 157,
            241, 164, 23, 85, 145, 11, 128, 66, 170, 101, 80, 5, 137, 86, 187, 249, 34, 70, 25,
            244, 29, 254, 209, 69, 6,
        ],
        [
            6, 172, 70, 82, 125, 132, 45, 219, 188, 58, 189, 244, 93, 103, 209, 245, 165, 188, 126,
            67, 216, 225, 157, 232, 230, 48, 208, 142, 155, 119, 152, 192, 27, 251, 248, 250, 202,
            202, 109, 61, 189, 155, 206, 64, 105, 23, 157, 91, 234, 7, 222, 21, 138, 2, 204, 42,
            183, 112, 87, 105, 2, 187, 243, 212,
        ],
        [
            46, 153, 248, 19, 244, 165, 105, 14, 78, 80, 199, 226, 192, 127, 80, 42, 33, 227, 234,
            72, 105, 237, 139, 222, 179, 219, 150, 232, 128, 114, 195, 229, 38, 241, 16, 105, 50,
            208, 255, 32, 226, 76, 157, 115, 70, 11, 243, 39, 70, 217, 197, 29, 169, 132, 15, 217,
            38, 172, 91, 44, 150, 105, 148, 81,
        ],
    ],
};
