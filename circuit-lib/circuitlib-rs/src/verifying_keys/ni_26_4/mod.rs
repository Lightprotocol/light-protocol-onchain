use groth16_solana::groth16::Groth16Verifyingkey;

pub const VERIFYINGKEY: Groth16Verifyingkey = Groth16Verifyingkey {
    nr_pubinputs: 9,

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
        10, 37, 112, 6, 255, 223, 131, 207, 228, 65, 121, 217, 20, 60, 183, 156, 3, 133, 71, 22,
        26, 24, 154, 84, 85, 191, 55, 124, 99, 161, 6, 18, 12, 182, 13, 127, 123, 153, 64, 189,
        212, 243, 22, 118, 159, 46, 195, 208, 222, 182, 236, 60, 250, 131, 97, 142, 192, 229, 130,
        98, 18, 47, 90, 84, 3, 101, 63, 110, 240, 139, 172, 87, 127, 245, 169, 138, 185, 62, 133,
        173, 225, 198, 39, 106, 87, 197, 15, 208, 6, 96, 146, 23, 22, 223, 82, 0, 46, 127, 116,
        140, 238, 9, 225, 239, 63, 233, 238, 175, 85, 124, 180, 227, 207, 57, 186, 186, 12, 87, 15,
        32, 225, 164, 114, 147, 237, 173, 7, 118,
    ],

    vk_ic: &[
        [
            22, 73, 169, 124, 24, 21, 4, 133, 34, 102, 228, 195, 95, 239, 117, 173, 34, 217, 68,
            20, 13, 72, 254, 62, 119, 133, 44, 87, 66, 28, 195, 106, 2, 7, 172, 120, 184, 239, 63,
            75, 229, 71, 159, 125, 174, 24, 35, 235, 118, 219, 175, 179, 218, 186, 185, 98, 63, 90,
            211, 113, 224, 70, 221, 143,
        ],
        [
            15, 185, 182, 178, 190, 39, 242, 182, 68, 47, 27, 233, 89, 225, 134, 160, 79, 96, 254,
            94, 93, 8, 200, 211, 215, 31, 14, 240, 64, 235, 82, 181, 13, 48, 115, 1, 89, 212, 79,
            129, 226, 177, 63, 114, 252, 252, 9, 40, 25, 162, 60, 141, 97, 18, 36, 226, 207, 220,
            155, 20, 115, 232, 228, 171,
        ],
        [
            12, 148, 24, 154, 161, 166, 207, 212, 255, 0, 180, 115, 200, 79, 210, 184, 81, 61, 165,
            61, 8, 126, 65, 255, 135, 194, 179, 162, 33, 214, 100, 141, 24, 249, 118, 246, 187, 84,
            234, 218, 140, 96, 173, 132, 241, 18, 184, 130, 137, 86, 68, 166, 243, 221, 207, 117,
            69, 213, 76, 226, 244, 18, 241, 15,
        ],
        [
            30, 207, 170, 68, 146, 134, 112, 92, 237, 22, 194, 136, 207, 91, 143, 37, 9, 62, 38,
            195, 197, 4, 109, 158, 4, 182, 227, 234, 209, 92, 37, 24, 38, 171, 100, 129, 82, 106,
            59, 65, 205, 91, 236, 0, 191, 66, 152, 120, 245, 201, 215, 148, 17, 244, 0, 158, 155,
            229, 8, 106, 65, 55, 92, 102,
        ],
        [
            36, 166, 214, 151, 47, 198, 147, 12, 77, 131, 34, 187, 197, 9, 25, 245, 109, 39, 149,
            151, 219, 30, 81, 85, 182, 72, 158, 201, 85, 137, 92, 138, 43, 71, 13, 250, 182, 4,
            202, 33, 249, 200, 54, 115, 15, 195, 26, 43, 208, 189, 183, 22, 2, 99, 9, 54, 47, 129,
            212, 49, 89, 33, 95, 231,
        ],
        [
            19, 51, 166, 223, 174, 130, 103, 188, 131, 182, 159, 86, 124, 56, 50, 32, 224, 191,
            157, 28, 218, 113, 186, 68, 150, 12, 117, 180, 128, 142, 40, 105, 24, 77, 31, 245, 90,
            202, 242, 170, 128, 186, 127, 184, 115, 230, 105, 223, 62, 47, 104, 237, 87, 209, 125,
            216, 221, 159, 192, 148, 156, 42, 188, 198,
        ],
        [
            34, 148, 36, 14, 201, 208, 105, 168, 128, 24, 182, 92, 32, 178, 151, 133, 88, 138, 236,
            134, 86, 185, 70, 230, 201, 93, 72, 43, 69, 198, 161, 51, 16, 60, 0, 235, 254, 245, 51,
            176, 53, 45, 44, 116, 142, 171, 19, 59, 225, 5, 90, 195, 115, 83, 102, 29, 211, 235,
            92, 161, 181, 41, 171, 130,
        ],
        [
            14, 90, 130, 0, 43, 226, 86, 29, 252, 20, 195, 90, 136, 204, 35, 199, 97, 125, 200, 22,
            101, 64, 15, 55, 90, 9, 140, 142, 253, 102, 148, 232, 11, 101, 197, 194, 113, 142, 217,
            164, 83, 197, 166, 31, 229, 208, 58, 170, 209, 86, 203, 147, 95, 152, 65, 97, 133, 122,
            179, 104, 244, 1, 153, 57,
        ],
        [
            37, 77, 91, 190, 238, 89, 172, 121, 155, 35, 78, 67, 128, 178, 55, 86, 222, 209, 49,
            61, 6, 186, 226, 25, 176, 241, 35, 132, 94, 155, 203, 202, 3, 21, 46, 139, 167, 102,
            143, 189, 157, 14, 90, 29, 165, 193, 230, 22, 228, 253, 91, 228, 95, 51, 38, 242, 106,
            162, 73, 30, 15, 186, 4, 48,
        ],
    ],
};
