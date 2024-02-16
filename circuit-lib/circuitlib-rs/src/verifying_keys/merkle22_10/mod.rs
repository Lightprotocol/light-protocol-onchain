use groth16_solana::groth16::Groth16Verifyingkey;

pub const VERIFYINGKEY: Groth16Verifyingkey = Groth16Verifyingkey {
    nr_pubinputs: 21,

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
        16, 249, 104, 56, 211, 161, 151, 0, 240, 191, 151, 101, 168, 151, 227, 62, 228, 130, 150,
        126, 26, 67, 181, 216, 88, 174, 29, 247, 184, 94, 132, 56, 24, 19, 114, 238, 174, 60, 22,
        71, 32, 220, 80, 183, 112, 216, 43, 52, 8, 230, 247, 46, 73, 42, 23, 30, 102, 166, 17, 163,
        19, 28, 195, 165, 13, 102, 223, 61, 18, 137, 103, 55, 124, 244, 199, 29, 51, 236, 66, 192,
        155, 176, 122, 24, 121, 144, 202, 189, 182, 140, 34, 224, 135, 232, 143, 186, 1, 193, 95,
        116, 140, 51, 23, 210, 23, 99, 19, 59, 156, 29, 228, 180, 19, 175, 83, 24, 128, 144, 236,
        58, 208, 81, 100, 115, 250, 172, 195, 214,
    ],

    vk_ic: &[
        [
            14, 40, 180, 247, 149, 68, 130, 170, 143, 234, 63, 33, 114, 19, 197, 101, 156, 154, 76,
            41, 227, 116, 216, 152, 195, 100, 80, 71, 113, 133, 58, 12, 25, 243, 130, 124, 133, 95,
            162, 211, 190, 109, 29, 122, 77, 22, 62, 253, 133, 103, 232, 222, 249, 141, 238, 189,
            96, 253, 56, 226, 5, 155, 48, 134,
        ],
        [
            21, 162, 80, 97, 19, 112, 69, 142, 185, 23, 96, 11, 164, 41, 71, 137, 70, 176, 162, 48,
            149, 139, 148, 65, 155, 29, 50, 122, 216, 194, 1, 107, 14, 20, 177, 132, 17, 151, 239,
            71, 7, 146, 62, 28, 231, 4, 161, 34, 164, 114, 88, 4, 206, 141, 26, 12, 14, 129, 47,
            218, 137, 210, 50, 6,
        ],
        [
            23, 238, 1, 229, 158, 11, 1, 204, 238, 209, 196, 27, 139, 142, 163, 19, 179, 113, 230,
            163, 188, 225, 58, 243, 110, 215, 240, 250, 54, 243, 218, 107, 14, 222, 91, 210, 242,
            169, 185, 69, 60, 40, 92, 77, 188, 189, 143, 84, 12, 76, 251, 171, 189, 14, 177, 28,
            248, 239, 203, 62, 5, 138, 38, 152,
        ],
        [
            46, 103, 149, 110, 196, 216, 91, 118, 235, 111, 74, 152, 187, 54, 119, 160, 182, 174,
            10, 143, 90, 186, 148, 0, 106, 63, 63, 141, 170, 233, 12, 11, 10, 233, 48, 52, 126, 4,
            160, 214, 218, 9, 50, 187, 235, 56, 201, 121, 27, 74, 113, 10, 111, 183, 206, 87, 103,
            190, 70, 42, 158, 124, 150, 16,
        ],
        [
            18, 171, 86, 19, 117, 245, 82, 90, 179, 20, 195, 222, 242, 104, 131, 161, 217, 180,
            250, 37, 6, 26, 81, 254, 222, 207, 45, 112, 171, 111, 62, 53, 0, 105, 139, 238, 210,
            242, 83, 58, 72, 85, 248, 141, 169, 200, 55, 236, 157, 209, 95, 205, 59, 178, 125, 122,
            120, 123, 222, 194, 43, 30, 55, 204,
        ],
        [
            40, 82, 133, 172, 123, 106, 91, 146, 106, 39, 17, 211, 228, 68, 192, 247, 58, 123, 208,
            35, 153, 10, 141, 78, 63, 130, 88, 54, 21, 237, 6, 249, 22, 228, 62, 224, 50, 5, 189,
            195, 157, 3, 29, 21, 138, 176, 68, 48, 186, 75, 10, 79, 253, 98, 128, 201, 245, 224,
            76, 19, 153, 27, 158, 223,
        ],
        [
            33, 237, 35, 64, 117, 215, 108, 162, 33, 40, 110, 8, 8, 109, 83, 134, 105, 106, 85,
            184, 53, 91, 29, 231, 243, 238, 83, 25, 185, 255, 52, 48, 14, 3, 239, 196, 137, 203,
            112, 57, 185, 35, 221, 16, 191, 183, 149, 176, 180, 24, 100, 74, 71, 154, 238, 28, 20,
            77, 212, 40, 46, 84, 166, 24,
        ],
        [
            12, 229, 9, 206, 26, 100, 29, 235, 232, 183, 74, 174, 158, 171, 174, 55, 17, 142, 182,
            87, 189, 12, 92, 234, 0, 77, 100, 146, 109, 94, 74, 130, 30, 146, 241, 14, 145, 239,
            123, 52, 69, 195, 67, 69, 180, 65, 139, 150, 2, 82, 165, 152, 223, 14, 20, 146, 126,
            32, 21, 237, 145, 13, 75, 189,
        ],
        [
            14, 138, 31, 230, 59, 72, 212, 223, 110, 27, 160, 52, 239, 129, 104, 6, 100, 49, 180,
            30, 153, 24, 58, 254, 92, 201, 62, 64, 111, 183, 28, 74, 12, 76, 135, 84, 87, 232, 214,
            66, 101, 99, 79, 129, 29, 148, 39, 211, 154, 28, 32, 113, 204, 198, 198, 202, 209, 210,
            86, 84, 68, 91, 39, 88,
        ],
        [
            35, 250, 106, 162, 59, 170, 117, 68, 14, 136, 166, 247, 207, 180, 187, 61, 231, 217,
            61, 103, 50, 104, 169, 91, 92, 63, 56, 118, 13, 74, 189, 177, 41, 20, 124, 9, 152, 192,
            143, 45, 62, 57, 143, 57, 217, 19, 80, 35, 22, 156, 221, 166, 114, 53, 3, 105, 32, 22,
            205, 162, 184, 123, 87, 94,
        ],
        [
            44, 161, 232, 150, 246, 234, 36, 167, 1, 192, 14, 106, 131, 55, 51, 92, 159, 24, 73,
            154, 193, 247, 62, 134, 160, 246, 221, 246, 107, 4, 174, 20, 35, 58, 121, 148, 117,
            118, 169, 128, 176, 65, 81, 252, 254, 139, 59, 211, 1, 233, 210, 231, 154, 170, 230,
            119, 93, 63, 155, 238, 36, 139, 132, 238,
        ],
        [
            25, 185, 35, 5, 144, 135, 115, 76, 102, 3, 22, 126, 100, 35, 129, 198, 149, 111, 46,
            19, 107, 90, 138, 241, 252, 72, 250, 128, 239, 156, 105, 122, 25, 123, 243, 152, 220,
            89, 78, 7, 130, 211, 218, 64, 11, 212, 253, 198, 29, 69, 230, 91, 190, 48, 235, 239,
            146, 219, 49, 48, 56, 120, 31, 91,
        ],
        [
            12, 181, 97, 4, 49, 218, 46, 247, 48, 140, 7, 242, 93, 150, 208, 68, 225, 179, 202,
            199, 181, 138, 38, 167, 4, 40, 6, 69, 51, 109, 138, 221, 1, 251, 119, 90, 108, 223,
            212, 112, 158, 183, 232, 194, 82, 191, 92, 32, 239, 174, 109, 238, 100, 165, 5, 137,
            173, 108, 138, 138, 163, 87, 47, 124,
        ],
        [
            5, 220, 53, 101, 35, 173, 54, 195, 255, 160, 162, 45, 221, 118, 35, 248, 234, 139, 108,
            88, 121, 8, 222, 230, 103, 76, 221, 42, 40, 227, 111, 151, 5, 235, 208, 178, 117, 10,
            239, 76, 133, 204, 217, 230, 132, 139, 180, 32, 208, 65, 212, 58, 210, 51, 23, 42, 120,
            104, 181, 131, 112, 133, 196, 136,
        ],
        [
            32, 136, 27, 43, 172, 104, 110, 205, 224, 38, 170, 127, 201, 117, 208, 248, 202, 115,
            112, 247, 74, 131, 42, 110, 202, 201, 16, 62, 5, 178, 231, 174, 12, 63, 158, 140, 62,
            247, 108, 201, 111, 253, 154, 131, 28, 81, 227, 233, 240, 231, 54, 212, 76, 238, 245,
            102, 32, 18, 97, 116, 13, 136, 156, 117,
        ],
        [
            35, 167, 176, 95, 178, 224, 106, 98, 12, 148, 25, 206, 56, 85, 214, 54, 110, 66, 58,
            86, 156, 116, 53, 45, 85, 227, 231, 134, 140, 51, 69, 103, 25, 4, 147, 190, 34, 58,
            179, 52, 176, 24, 251, 100, 29, 76, 202, 72, 239, 143, 54, 137, 167, 164, 47, 240, 153,
            31, 197, 42, 48, 162, 51, 106,
        ],
        [
            12, 215, 207, 129, 1, 37, 242, 174, 117, 124, 200, 214, 136, 128, 199, 241, 6, 147,
            190, 133, 120, 144, 66, 145, 4, 103, 197, 33, 179, 84, 81, 37, 1, 104, 124, 44, 24,
            183, 161, 221, 128, 87, 120, 36, 69, 70, 137, 227, 30, 208, 204, 179, 38, 53, 219, 81,
            180, 71, 173, 242, 179, 243, 81, 38,
        ],
        [
            23, 110, 239, 69, 3, 133, 225, 239, 28, 162, 60, 33, 62, 196, 161, 182, 127, 29, 191,
            179, 222, 236, 209, 79, 92, 83, 224, 169, 144, 219, 202, 158, 32, 91, 18, 27, 31, 100,
            220, 76, 93, 188, 151, 213, 63, 139, 13, 182, 164, 119, 77, 37, 145, 54, 142, 186, 111,
            81, 121, 140, 9, 252, 159, 135,
        ],
        [
            41, 27, 14, 61, 21, 19, 27, 246, 252, 249, 163, 141, 2, 142, 90, 216, 238, 48, 30, 65,
            210, 86, 122, 197, 128, 155, 181, 51, 181, 64, 210, 91, 21, 7, 15, 110, 145, 15, 71,
            108, 38, 130, 213, 139, 17, 196, 12, 79, 175, 84, 210, 237, 127, 24, 223, 15, 245, 134,
            27, 99, 173, 239, 139, 224,
        ],
        [
            35, 169, 242, 157, 213, 138, 52, 230, 237, 173, 117, 139, 134, 124, 17, 47, 100, 129,
            197, 199, 22, 24, 174, 76, 116, 207, 22, 27, 30, 228, 126, 184, 19, 3, 178, 77, 28,
            232, 59, 13, 169, 44, 177, 123, 68, 103, 12, 0, 13, 196, 181, 237, 141, 143, 252, 49,
            109, 248, 61, 88, 240, 169, 77, 37,
        ],
        [
            34, 139, 243, 184, 191, 44, 60, 114, 55, 123, 190, 55, 249, 190, 36, 140, 62, 177, 229,
            112, 15, 105, 134, 110, 113, 127, 148, 180, 246, 126, 65, 201, 2, 137, 121, 70, 190,
            217, 40, 170, 234, 62, 7, 152, 216, 118, 44, 116, 91, 71, 77, 140, 72, 59, 97, 241,
            120, 68, 131, 46, 225, 244, 152, 173,
        ],
    ],
};
