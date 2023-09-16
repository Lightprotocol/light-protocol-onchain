export const IDL = {
    "version": "0.1.0",
    "name": "verifier_program_two",
    "constants": [
        {
            "name": "PROGRAM_ID",
            "type": "string",
            "value": "\"2cxC8e8uNYLcymH6RTGuJs3N8fXGkwmMpw45pY65Ay86\""
        },
        {
            "name": "VERIFYINGKEY",
            "type": {
                "defined": "Groth16Verifyingkey"
            },
            "value": "Groth16Verifyingkey { nr_pubinputs : 16 , vk_alpha_g1 : [45 , 77 , 154 , 167 , 227 , 2 , 217 , 223 , 65 , 116 , 157 , 85 , 7 , 148 , 157 , 5 , 219 , 234 , 51 , 251 , 177 , 108 , 100 , 59 , 34 , 245 , 153 , 162 , 190 , 109 , 242 , 226 , 20 , 190 , 221 , 80 , 60 , 55 , 206 , 176 , 97 , 216 , 236 , 96 , 32 , 159 , 227 , 69 , 206 , 137 , 131 , 10 , 25 , 35 , 3 , 1 , 240 , 118 , 202 , 255 , 0 , 77 , 25 , 38] , vk_beta_g2 : [9 , 103 , 3 , 47 , 203 , 247 , 118 , 209 , 175 , 201 , 133 , 248 , 136 , 119 , 241 , 130 , 211 , 132 , 128 , 166 , 83 , 242 , 222 , 202 , 169 , 121 , 76 , 188 , 59 , 243 , 6 , 12 , 14 , 24 , 120 , 71 , 173 , 76 , 121 , 131 , 116 , 208 , 214 , 115 , 43 , 245 , 1 , 132 , 125 , 214 , 139 , 192 , 224 , 113 , 36 , 30 , 2 , 19 , 188 , 127 , 193 , 61 , 183 , 171 , 48 , 76 , 251 , 209 , 224 , 138 , 112 , 74 , 153 , 245 , 232 , 71 , 217 , 63 , 140 , 60 , 170 , 253 , 222 , 196 , 107 , 122 , 13 , 55 , 157 , 166 , 154 , 77 , 17 , 35 , 70 , 167 , 23 , 57 , 193 , 177 , 164 , 87 , 168 , 199 , 49 , 49 , 35 , 210 , 77 , 47 , 145 , 146 , 248 , 150 , 183 , 198 , 62 , 234 , 5 , 169 , 213 , 127 , 6 , 84 , 122 , 208 , 206 , 200] , vk_gamme_g2 : [25 , 142 , 147 , 147 , 146 , 13 , 72 , 58 , 114 , 96 , 191 , 183 , 49 , 251 , 93 , 37 , 241 , 170 , 73 , 51 , 53 , 169 , 231 , 18 , 151 , 228 , 133 , 183 , 174 , 243 , 18 , 194 , 24 , 0 , 222 , 239 , 18 , 31 , 30 , 118 , 66 , 106 , 0 , 102 , 94 , 92 , 68 , 121 , 103 , 67 , 34 , 212 , 247 , 94 , 218 , 221 , 70 , 222 , 189 , 92 , 217 , 146 , 246 , 237 , 9 , 6 , 137 , 208 , 88 , 95 , 240 , 117 , 236 , 158 , 153 , 173 , 105 , 12 , 51 , 149 , 188 , 75 , 49 , 51 , 112 , 179 , 142 , 243 , 85 , 172 , 218 , 220 , 209 , 34 , 151 , 91 , 18 , 200 , 94 , 165 , 219 , 140 , 109 , 235 , 74 , 171 , 113 , 128 , 141 , 203 , 64 , 143 , 227 , 209 , 231 , 105 , 12 , 67 , 211 , 123 , 76 , 230 , 204 , 1 , 102 , 250 , 125 , 170] , vk_delta_g2 : [28 , 218 , 18 , 116 , 42 , 25 , 193 , 26 , 116 , 120 , 0 , 22 , 47 , 167 , 224 , 167 , 98 , 90 , 70 , 211 , 6 , 97 , 75 , 105 , 145 , 215 , 29 , 157 , 252 , 163 , 9 , 62 , 42 , 188 , 70 , 184 , 37 , 134 , 211 , 9 , 35 , 165 , 191 , 77 , 138 , 204 , 106 , 193 , 43 , 226 , 112 , 254 , 135 , 214 , 6 , 6 , 225 , 186 , 151 , 32 , 94 , 194 , 103 , 251 , 32 , 122 , 114 , 253 , 226 , 158 , 102 , 64 , 96 , 190 , 33 , 208 , 201 , 77 , 253 , 221 , 235 , 13 , 135 , 30 , 9 , 56 , 151 , 74 , 51 , 172 , 7 , 202 , 107 , 114 , 163 , 139 , 1 , 63 , 190 , 255 , 202 , 238 , 20 , 230 , 118 , 208 , 178 , 35 , 217 , 210 , 67 , 61 , 178 , 132 , 11 , 102 , 248 , 10 , 121 , 176 , 231 , 81 , 3 , 215 , 170 , 212 , 41 , 206] , vk_ic : & [[34 , 230 , 106 , 80 , 130 , 180 , 158 , 65 , 219 , 192 , 178 , 139 , 7 , 80 , 242 , 223 , 102 , 23 , 202 , 78 , 1 , 244 , 159 , 136 , 1 , 219 , 2 , 42 , 31 , 205 , 52 , 133 , 32 , 164 , 225 , 71 , 101 , 233 , 119 , 156 , 104 , 113 , 19 , 141 , 220 , 219 , 91 , 92 , 195 , 183 , 10 , 193 , 114 , 169 , 9 , 89 , 32 , 65 , 12 , 134 , 147 , 90 , 229 , 36] , [25 , 176 , 118 , 184 , 36 , 230 , 94 , 84 , 116 , 102 , 124 , 103 , 115 , 225 , 64 , 250 , 26 , 148 , 59 , 198 , 98 , 185 , 105 , 53 , 171 , 37 , 61 , 28 , 177 , 154 , 187 , 228 , 21 , 250 , 186 , 169 , 100 , 208 , 38 , 64 , 249 , 217 , 246 , 171 , 209 , 111 , 255 , 158 , 90 , 120 , 19 , 35 , 224 , 248 , 165 , 240 , 189 , 135 , 18 , 187 , 35 , 96 , 203 , 27] , [38 , 197 , 141 , 43 , 251 , 79 , 74 , 188 , 24 , 71 , 95 , 100 , 168 , 37 , 50 , 204 , 41 , 93 , 49 , 159 , 148 , 223 , 100 , 9 , 0 , 110 , 75 , 22 , 147 , 64 , 221 , 250 , 20 , 87 , 89 , 63 , 99 , 13 , 194 , 82 , 104 , 56 , 193 , 147 , 51 , 157 , 45 , 167 , 49 , 54 , 45 , 163 , 78 , 22 , 23 , 174 , 82 , 42 , 245 , 76 , 226 , 154 , 114 , 76] , [24 , 102 , 230 , 196 , 85 , 76 , 215 , 123 , 112 , 160 , 183 , 220 , 140 , 238 , 90 , 110 , 103 , 243 , 38 , 169 , 61 , 37 , 61 , 213 , 48 , 178 , 173 , 52 , 63 , 74 , 37 , 202 , 39 , 73 , 221 , 124 , 203 , 48 , 153 , 136 , 94 , 135 , 169 , 27 , 213 , 152 , 225 , 100 , 168 , 50 , 47 , 106 , 242 , 254 , 67 , 10 , 77 , 127 , 141 , 10 , 28 , 197 , 52 , 214] , [5 , 109 , 57 , 52 , 181 , 39 , 129 , 207 , 175 , 255 , 168 , 234 , 113 , 216 , 231 , 35 , 222 , 143 , 107 , 178 , 212 , 43 , 141 , 244 , 59 , 101 , 91 , 104 , 170 , 226 , 62 , 255 , 38 , 243 , 73 , 116 , 235 , 132 , 54 , 242 , 116 , 89 , 248 , 47 , 66 , 154 , 122 , 169 , 189 , 8 , 233 , 107 , 180 , 27 , 96 , 110 , 135 , 41 , 17 , 157 , 168 , 11 , 157 , 51] , [47 , 116 , 175 , 9 , 152 , 228 , 74 , 196 , 184 , 206 , 190 , 22 , 138 , 233 , 86 , 208 , 154 , 139 , 14 , 72 , 196 , 186 , 80 , 253 , 68 , 253 , 121 , 208 , 131 , 195 , 219 , 69 , 32 , 52 , 195 , 16 , 132 , 121 , 0 , 79 , 228 , 54 , 24 , 78 , 214 , 151 , 204 , 252 , 30 , 45 , 81 , 155 , 127 , 71 , 216 , 37 , 230 , 253 , 210 , 72 , 118 , 95 , 6 , 58] , [41 , 103 , 120 , 254 , 133 , 212 , 94 , 190 , 146 , 200 , 195 , 217 , 67 , 163 , 226 , 195 , 130 , 124 , 206 , 235 , 163 , 157 , 23 , 3 , 47 , 60 , 198 , 232 , 6 , 189 , 15 , 154 , 39 , 248 , 74 , 71 , 89 , 189 , 224 , 220 , 81 , 36 , 141 , 12 , 16 , 98 , 94 , 117 , 129 , 13 , 85 , 84 , 107 , 50 , 228 , 146 , 80 , 252 , 250 , 104 , 14 , 54 , 182 , 145] , [34 , 113 , 2 , 10 , 65 , 130 , 58 , 182 , 105 , 63 , 85 , 87 , 19 , 119 , 14 , 55 , 225 , 85 , 211 , 120 , 53 , 97 , 191 , 160 , 118 , 114 , 13 , 222 , 120 , 139 , 202 , 141 , 37 , 38 , 193 , 14 , 61 , 65 , 95 , 9 , 19 , 183 , 14 , 154 , 221 , 195 , 230 , 87 , 145 , 102 , 252 , 8 , 234 , 31 , 74 , 93 , 7 , 80 , 168 , 244 , 136 , 45 , 62 , 230] , [20 , 230 , 189 , 106 , 32 , 36 , 82 , 119 , 108 , 235 , 41 , 3 , 156 , 174 , 32 , 103 , 234 , 56 , 179 , 42 , 41 , 40 , 94 , 173 , 162 , 220 , 162 , 214 , 140 , 20 , 202 , 139 , 29 , 72 , 130 , 147 , 124 , 80 , 55 , 171 , 143 , 126 , 55 , 150 , 52 , 149 , 27 , 100 , 234 , 190 , 132 , 7 , 183 , 24 , 162 , 158 , 193 , 65 , 222 , 174 , 123 , 47 , 157 , 145] , [14 , 234 , 77 , 133 , 167 , 110 , 59 , 67 , 112 , 181 , 50 , 167 , 166 , 112 , 222 , 114 , 175 , 142 , 54 , 184 , 225 , 38 , 214 , 119 , 178 , 201 , 172 , 47 , 1 , 145 , 101 , 144 , 36 , 26 , 20 , 119 , 247 , 212 , 98 , 40 , 66 , 228 , 146 , 53 , 9 , 81 , 39 , 186 , 181 , 69 , 71 , 177 , 71 , 13 , 96 , 128 , 222 , 238 , 126 , 193 , 98 , 204 , 137 , 180] , [27 , 176 , 142 , 187 , 232 , 16 , 152 , 44 , 166 , 125 , 227 , 72 , 75 , 18 , 29 , 11 , 77 , 203 , 103 , 15 , 102 , 18 , 11 , 201 , 118 , 174 , 225 , 38 , 241 , 11 , 184 , 112 , 1 , 80 , 166 , 35 , 244 , 134 , 101 , 176 , 35 , 208 , 166 , 147 , 246 , 114 , 160 , 158 , 7 , 66 , 56 , 169 , 204 , 3 , 19 , 132 , 39 , 92 , 13 , 33 , 12 , 15 , 44 , 221] , [26 , 243 , 97 , 157 , 24 , 32 , 156 , 93 , 55 , 75 , 119 , 174 , 218 , 66 , 163 , 73 , 58 , 46 , 235 , 100 , 120 , 202 , 56 , 122 , 36 , 171 , 234 , 113 , 220 , 197 , 95 , 73 , 28 , 255 , 129 , 1 , 139 , 31 , 45 , 123 , 0 , 250 , 66 , 83 , 36 , 110 , 14 , 194 , 19 , 202 , 213 , 99 , 235 , 96 , 233 , 22 , 36 , 206 , 202 , 25 , 115 , 156 , 216 , 100] , [15 , 66 , 125 , 98 , 250 , 108 , 60 , 160 , 68 , 151 , 0 , 196 , 26 , 105 , 99 , 195 , 212 , 250 , 62 , 245 , 58 , 37 , 142 , 29 , 70 , 45 , 177 , 253 , 194 , 67 , 251 , 33 , 18 , 46 , 13 , 227 , 76 , 126 , 217 , 83 , 244 , 75 , 135 , 52 , 200 , 132 , 97 , 59 , 95 , 112 , 66 , 78 , 83 , 11 , 246 , 230 , 132 , 5 , 81 , 77 , 224 , 106 , 122 , 215] , [43 , 210 , 103 , 108 , 46 , 86 , 44 , 186 , 226 , 83 , 78 , 230 , 59 , 141 , 112 , 134 , 123 , 54 , 254 , 207 , 69 , 145 , 141 , 59 , 204 , 86 , 126 , 195 , 70 , 192 , 116 , 9 , 20 , 219 , 238 , 209 , 147 , 207 , 45 , 148 , 124 , 170 , 98 , 84 , 12 , 49 , 216 , 52 , 2 , 59 , 185 , 127 , 41 , 190 , 76 , 107 , 13 , 88 , 172 , 76 , 134 , 36 , 210 , 195] , [8 , 92 , 34 , 119 , 24 , 224 , 145 , 85 , 179 , 50 , 68 , 131 , 135 , 91 , 74 , 68 , 72 , 192 , 164 , 206 , 225 , 123 , 6 , 134 , 173 , 204 , 153 , 225 , 142 , 230 , 192 , 36 , 15 , 30 , 162 , 218 , 222 , 197 , 47 , 251 , 245 , 228 , 219 , 25 , 156 , 88 , 146 , 236 , 44 , 9 , 96 , 29 , 120 , 115 , 174 , 53 , 215 , 87 , 45 , 96 , 33 , 50 , 164 , 142] , [2 , 23 , 98 , 199 , 152 , 140 , 91 , 106 , 78 , 245 , 44 , 110 , 217 , 48 , 115 , 44 , 103 , 213 , 204 , 19 , 176 , 24 , 223 , 102 , 75 , 230 , 90 , 213 , 145 , 110 , 244 , 41 , 37 , 112 , 5 , 195 , 30 , 43 , 117 , 149 , 208 , 36 , 224 , 214 , 190 , 83 , 48 , 186 , 213 , 63 , 40 , 209 , 89 , 234 , 58 , 160 , 183 , 66 , 60 , 87 , 218 , 78 , 81 , 230]] , }"
        }
    ],
    "instructions": [
        {
            "name": "shieldedTransferInputs",
            "docs": [
                "This instruction is used to invoke this system verifier and can only be invoked via cpi."
            ],
            "accounts": [
                {
                    "name": "signingAddress",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "programMerkleTree",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "transactionMerkleTree",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "relayerRecipientSol",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "senderSol",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "recipientSol",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenAuthority",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "senderSpl",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "recipientSpl",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "registeredVerifierPda",
                    "isMut": true,
                    "isSigner": false,
                    "docs": [
                        "Verifier config pda which needs to exist."
                    ]
                },
                {
                    "name": "logWrapper",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "eventMerkleTree",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "verifierState",
                    "isMut": false,
                    "isSigner": true
                }
            ],
            "args": [
                {
                    "name": "proofA",
                    "type": {
                        "array": [
                            "u8",
                            64
                        ]
                    }
                },
                {
                    "name": "proofB",
                    "type": {
                        "array": [
                            "u8",
                            128
                        ]
                    }
                },
                {
                    "name": "proofC",
                    "type": {
                        "array": [
                            "u8",
                            64
                        ]
                    }
                },
                {
                    "name": "connectingHash",
                    "type": {
                        "array": [
                            "u8",
                            32
                        ]
                    }
                }
            ]
        }
    ],
    "accounts": [
        {
            "name": "zKtransactionApp4MainProofInputs",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "root",
                        "type": "u8"
                    },
                    {
                        "name": "publicAmountSpl",
                        "type": "u8"
                    },
                    {
                        "name": "txIntegrityHash",
                        "type": "u8"
                    },
                    {
                        "name": "publicAmountSol",
                        "type": "u8"
                    },
                    {
                        "name": "publicMintPubkey",
                        "type": "u8"
                    },
                    {
                        "name": "inputNullifier",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "outputCommitment",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "publicAppVerifier",
                        "type": "u8"
                    },
                    {
                        "name": "transactionHash",
                        "type": "u8"
                    },
                    {
                        "name": "inAmount",
                        "type": {
                            "array": [
                                {
                                    "array": [
                                        "u8",
                                        2
                                    ]
                                },
                                4
                            ]
                        }
                    },
                    {
                        "name": "inPrivateKey",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "inBlinding",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "inAppDataHash",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "inPoolType",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "inVerifierPubkey",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "inPathIndices",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "inPathElements",
                        "type": {
                            "array": [
                                {
                                    "array": [
                                        "u8",
                                        18
                                    ]
                                },
                                4
                            ]
                        }
                    },
                    {
                        "name": "inIndices",
                        "type": {
                            "array": [
                                {
                                    "array": [
                                        {
                                            "array": [
                                                "u8",
                                                3
                                            ]
                                        },
                                        2
                                    ]
                                },
                                4
                            ]
                        }
                    },
                    {
                        "name": "outAmount",
                        "type": {
                            "array": [
                                {
                                    "array": [
                                        "u8",
                                        2
                                    ]
                                },
                                4
                            ]
                        }
                    },
                    {
                        "name": "outPubkey",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "outBlinding",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "outAppDataHash",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "outIndices",
                        "type": {
                            "array": [
                                {
                                    "array": [
                                        {
                                            "array": [
                                                "u8",
                                                3
                                            ]
                                        },
                                        2
                                    ]
                                },
                                4
                            ]
                        }
                    },
                    {
                        "name": "outPoolType",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "outVerifierPubkey",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "assetPubkeys",
                        "type": {
                            "array": [
                                "u8",
                                3
                            ]
                        }
                    },
                    {
                        "name": "transactionVersion",
                        "type": "u8"
                    },
                    {
                        "name": "internalTxIntegrityHash",
                        "type": "u8"
                    }
                ]
            }
        },
        {
            "name": "zKtransactionApp4MainPublicInputs",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "root",
                        "type": "u8"
                    },
                    {
                        "name": "publicAmountSpl",
                        "type": "u8"
                    },
                    {
                        "name": "txIntegrityHash",
                        "type": "u8"
                    },
                    {
                        "name": "publicAmountSol",
                        "type": "u8"
                    },
                    {
                        "name": "publicMintPubkey",
                        "type": "u8"
                    },
                    {
                        "name": "inputNullifier",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "outputCommitment",
                        "type": {
                            "array": [
                                "u8",
                                4
                            ]
                        }
                    },
                    {
                        "name": "publicAppVerifier",
                        "type": "u8"
                    },
                    {
                        "name": "transactionHash",
                        "type": "u8"
                    }
                ]
            }
        }
    ],
    "errors": [
        {
            "code": 6000,
            "name": "InvalidVerifier",
            "msg": "System program is no valid verifier."
        }
    ]
};
//# sourceMappingURL=verifier_program_two.js.map