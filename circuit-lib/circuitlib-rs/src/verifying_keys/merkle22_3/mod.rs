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
        7, 61, 16, 190, 160, 208, 240, 218, 23, 188, 118, 7, 165, 249, 193, 66, 234, 136, 216, 247,
        165, 112, 166, 0, 4, 219, 7, 93, 165, 54, 81, 221, 18, 6, 136, 90, 207, 15, 69, 43, 85,
        177, 36, 105, 49, 111, 233, 69, 127, 113, 59, 90, 59, 26, 192, 122, 75, 65, 127, 69, 217,
        83, 0, 123, 15, 67, 143, 174, 74, 111, 144, 61, 168, 25, 75, 47, 35, 126, 87, 241, 199, 62,
        84, 133, 112, 239, 168, 158, 76, 181, 158, 185, 138, 95, 174, 40, 20, 226, 151, 129, 20,
        21, 251, 61, 199, 220, 0, 92, 122, 143, 109, 31, 122, 85, 88, 57, 222, 225, 71, 72, 200,
        102, 201, 52, 220, 156, 1, 52,
    ],

    vk_ic: &[
        [
            1, 158, 106, 188, 157, 171, 213, 33, 222, 67, 242, 159, 62, 120, 243, 59, 110, 240,
            198, 221, 48, 34, 57, 243, 20, 10, 254, 181, 166, 217, 72, 25, 31, 5, 147, 24, 98, 122,
            149, 207, 145, 73, 215, 101, 61, 186, 224, 249, 37, 212, 44, 137, 233, 2, 28, 135, 210,
            86, 184, 215, 204, 232, 236, 68,
        ],
        [
            8, 202, 169, 50, 63, 187, 224, 82, 158, 242, 238, 46, 14, 218, 78, 31, 1, 76, 127, 31,
            7, 67, 115, 49, 19, 194, 84, 212, 31, 71, 226, 106, 3, 159, 231, 57, 80, 110, 4, 158,
            5, 152, 34, 179, 204, 231, 90, 56, 212, 95, 86, 9, 117, 229, 144, 165, 175, 163, 150,
            186, 186, 200, 184, 13,
        ],
        [
            6, 113, 153, 231, 253, 40, 10, 246, 251, 149, 86, 108, 52, 104, 131, 246, 160, 182, 72,
            96, 37, 106, 117, 35, 134, 119, 218, 209, 199, 186, 95, 210, 39, 161, 192, 97, 52, 202,
            77, 58, 168, 36, 18, 137, 48, 4, 204, 70, 94, 173, 112, 190, 224, 246, 82, 174, 75, 41,
            242, 207, 133, 201, 20, 171,
        ],
        [
            23, 28, 114, 20, 43, 165, 147, 221, 94, 65, 162, 248, 24, 64, 255, 214, 227, 199, 51,
            61, 123, 242, 196, 18, 207, 205, 90, 102, 83, 52, 241, 152, 30, 51, 216, 68, 171, 109,
            213, 245, 243, 85, 43, 226, 249, 192, 125, 192, 152, 180, 111, 128, 26, 5, 65, 0, 144,
            249, 74, 156, 114, 38, 196, 216,
        ],
        [
            38, 94, 229, 102, 239, 236, 85, 104, 131, 113, 177, 18, 35, 23, 82, 56, 229, 213, 25,
            25, 102, 177, 55, 138, 160, 200, 63, 238, 138, 222, 245, 243, 36, 188, 87, 15, 199,
            104, 179, 153, 174, 177, 160, 207, 27, 201, 194, 212, 76, 205, 191, 237, 102, 10, 250,
            2, 119, 254, 6, 246, 157, 18, 27, 218,
        ],
        [
            40, 117, 11, 52, 88, 130, 92, 252, 15, 32, 174, 201, 173, 67, 69, 245, 191, 221, 217,
            0, 46, 40, 112, 133, 138, 110, 132, 89, 73, 103, 195, 211, 4, 136, 39, 28, 247, 22, 68,
            164, 244, 2, 73, 156, 229, 104, 173, 74, 10, 53, 195, 38, 117, 117, 74, 10, 78, 58,
            103, 120, 7, 79, 157, 211,
        ],
        [
            7, 209, 154, 41, 19, 90, 213, 131, 168, 189, 139, 28, 140, 211, 99, 56, 211, 122, 44,
            17, 116, 164, 157, 216, 1, 125, 52, 39, 36, 10, 137, 127, 26, 57, 142, 189, 67, 162,
            220, 222, 97, 109, 76, 173, 133, 153, 96, 216, 229, 177, 214, 120, 219, 66, 54, 34,
            178, 124, 68, 147, 177, 243, 46, 138,
        ],
    ],
};
