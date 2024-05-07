export type LightCompressedPda = {
    version: '0.3.1';
    name: 'light_compressed_pda';
    constants: [
        {
            name: 'COMPRESSED_SOL_PDA_SEED';
            type: 'bytes';
            value: '[99, 111, 109, 112, 114, 101, 115, 115, 101, 100, 95, 115, 111, 108, 95, 112, 100, 97]';
        },
    ];
    instructions: [
        {
            name: 'initCpiContextAccount';
            accounts: [
                {
                    name: 'feePayer';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'cpiContextAccount';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'systemProgram';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'associatedMerkleTree';
                    isMut: false;
                    isSigner: false;
                },
            ];
            args: [];
        },
        {
            name: 'invoke';
            accounts: [
                {
                    name: 'feePayer';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'authority';
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'noopProgram';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'accountCompressionAuthority';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'accountCompressionProgram';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'compressedSolPda';
                    isMut: true;
                    isSigner: false;
                    isOptional: true;
                },
                {
                    name: 'compressionRecipient';
                    isMut: true;
                    isSigner: false;
                    isOptional: true;
                },
                {
                    name: 'systemProgram';
                    isMut: false;
                    isSigner: false;
                },
            ];
            args: [
                {
                    name: 'inputs';
                    type: 'bytes';
                },
            ];
        },
        {
            name: 'invokeCpi';
            accounts: [
                {
                    name: 'feePayer';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'authority';
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'noopProgram';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'accountCompressionAuthority';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'accountCompressionProgram';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'invokingProgram';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'compressedSolPda';
                    isMut: true;
                    isSigner: false;
                    isOptional: true;
                },
                {
                    name: 'compressionRecipient';
                    isMut: true;
                    isSigner: false;
                    isOptional: true;
                },
                {
                    name: 'systemProgram';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'cpiContextAccount';
                    isMut: true;
                    isSigner: false;
                    isOptional: true;
                },
            ];
            args: [
                {
                    name: 'inputs';
                    type: 'bytes';
                },
            ];
        },
    ];
    accounts: [
        {
            name: 'compressedSolPda';
            type: {
                kind: 'struct';
                fields: [];
            };
        },
        {
            name: 'cpiContextAccount';
            docs: [
                'collects invocations without proofs',
                'invocations are collected and processed when an invocation with a proof is received',
            ];
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'associatedMerkleTree';
                        type: 'publicKey';
                    },
                    {
                        name: 'context';
                        type: {
                            vec: {
                                defined: 'InstructionDataInvokeCpi';
                            };
                        };
                    },
                ];
            };
        },
    ];
    types: [
        {
            name: 'InstructionDataInvoke';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'proof';
                        type: {
                            option: {
                                defined: 'CompressedProof';
                            };
                        };
                    },
                    {
                        name: 'inputRootIndices';
                        type: {
                            vec: 'u16';
                        };
                    },
                    {
                        name: 'inputCompressedAccountsWithMerkleContext';
                        type: {
                            vec: {
                                defined: 'PackedCompressedAccountWithMerkleContext';
                            };
                        };
                    },
                    {
                        name: 'outputCompressedAccounts';
                        type: {
                            vec: {
                                defined: 'CompressedAccount';
                            };
                        };
                    },
                    {
                        name: 'outputStateMerkleTreeAccountIndices';
                        docs: [
                            'The indices of the accounts in the output state merkle tree.',
                        ];
                        type: 'bytes';
                    },
                    {
                        name: 'relayFee';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'newAddressParams';
                        type: {
                            vec: {
                                defined: 'NewAddressParamsPacked';
                            };
                        };
                    },
                    {
                        name: 'compressionLamports';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'isCompress';
                        type: 'bool';
                    },
                ];
            };
        },
        {
            name: 'NewAddressParamsPacked';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'seed';
                        type: {
                            array: ['u8', 32];
                        };
                    },
                    {
                        name: 'addressQueueAccountIndex';
                        type: 'u8';
                    },
                    {
                        name: 'addressMerkleTreeAccountIndex';
                        type: 'u8';
                    },
                    {
                        name: 'addressMerkleTreeRootIndex';
                        type: 'u16';
                    },
                ];
            };
        },
        {
            name: 'NewAddressParams';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'seed';
                        type: {
                            array: ['u8', 32];
                        };
                    },
                    {
                        name: 'addressQueuePubkey';
                        type: 'publicKey';
                    },
                    {
                        name: 'addressMerkleTreePubkey';
                        type: 'publicKey';
                    },
                    {
                        name: 'addressMerkleTreeRootIndex';
                        type: 'u16';
                    },
                ];
            };
        },
        {
            name: 'CompressedProof';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'a';
                        type: {
                            array: ['u8', 32];
                        };
                    },
                    {
                        name: 'b';
                        type: {
                            array: ['u8', 64];
                        };
                    },
                    {
                        name: 'c';
                        type: {
                            array: ['u8', 32];
                        };
                    },
                ];
            };
        },
        {
            name: 'InstructionDataInvokeCpi';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'proof';
                        type: {
                            option: {
                                defined: 'CompressedProof';
                            };
                        };
                    },
                    {
                        name: 'newAddressParams';
                        type: {
                            vec: {
                                defined: 'NewAddressParamsPacked';
                            };
                        };
                    },
                    {
                        name: 'inputRootIndices';
                        type: {
                            vec: 'u16';
                        };
                    },
                    {
                        name: 'inputCompressedAccountsWithMerkleContext';
                        type: {
                            vec: {
                                defined: 'PackedCompressedAccountWithMerkleContext';
                            };
                        };
                    },
                    {
                        name: 'outputCompressedAccounts';
                        type: {
                            vec: {
                                defined: 'CompressedAccount';
                            };
                        };
                    },
                    {
                        name: 'outputStateMerkleTreeAccountIndices';
                        docs: [
                            'The indices of the accounts in the output state merkle tree.',
                        ];
                        type: 'bytes';
                    },
                    {
                        name: 'relayFee';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'compressionLamports';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'isCompress';
                        type: 'bool';
                    },
                    {
                        name: 'signerSeeds';
                        type: {
                            vec: 'bytes';
                        };
                    },
                    {
                        name: 'cpiContext';
                        type: {
                            option: {
                                defined: 'CompressedCpiContext';
                            };
                        };
                    },
                ];
            };
        },
        {
            name: 'PackedCompressedAccountWithMerkleContext';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'compressedAccount';
                        type: {
                            defined: 'CompressedAccount';
                        };
                    },
                    {
                        name: 'merkleContext';
                        type: {
                            defined: 'PackedMerkleContext';
                        };
                    },
                ];
            };
        },
        {
            name: 'CompressedAccountWithMerkleContext';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'compressedAccount';
                        type: {
                            defined: 'CompressedAccount';
                        };
                    },
                    {
                        name: 'merkleContext';
                        type: {
                            defined: 'MerkleContext';
                        };
                    },
                ];
            };
        },
        {
            name: 'MerkleContext';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'merkleTreePubkey';
                        type: 'publicKey';
                    },
                    {
                        name: 'nullifierQueuePubkey';
                        type: 'publicKey';
                    },
                    {
                        name: 'leafIndex';
                        type: 'u32';
                    },
                ];
            };
        },
        {
            name: 'PackedMerkleContext';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'merkleTreePubkeyIndex';
                        type: 'u8';
                    },
                    {
                        name: 'nullifierQueuePubkeyIndex';
                        type: 'u8';
                    },
                    {
                        name: 'leafIndex';
                        type: 'u32';
                    },
                ];
            };
        },
        {
            name: 'CompressedAccount';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'owner';
                        type: 'publicKey';
                    },
                    {
                        name: 'lamports';
                        type: 'u64';
                    },
                    {
                        name: 'address';
                        type: {
                            option: {
                                array: ['u8', 32];
                            };
                        };
                    },
                    {
                        name: 'data';
                        type: {
                            option: {
                                defined: 'CompressedAccountData';
                            };
                        };
                    },
                ];
            };
        },
        {
            name: 'CompressedAccountData';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'discriminator';
                        type: {
                            array: ['u8', 8];
                        };
                    },
                    {
                        name: 'data';
                        type: 'bytes';
                    },
                    {
                        name: 'dataHash';
                        type: {
                            array: ['u8', 32];
                        };
                    },
                ];
            };
        },
        {
            name: 'PublicTransactionEvent';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'inputCompressedAccountHashes';
                        type: {
                            vec: {
                                array: ['u8', 32];
                            };
                        };
                    },
                    {
                        name: 'outputCompressedAccountHashes';
                        type: {
                            vec: {
                                array: ['u8', 32];
                            };
                        };
                    },
                    {
                        name: 'outputCompressedAccounts';
                        type: {
                            vec: {
                                defined: 'CompressedAccount';
                            };
                        };
                    },
                    {
                        name: 'outputStateMerkleTreeAccountIndices';
                        type: 'bytes';
                    },
                    {
                        name: 'outputLeafIndices';
                        type: {
                            vec: 'u32';
                        };
                    },
                    {
                        name: 'relayFee';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'isCompress';
                        type: 'bool';
                    },
                    {
                        name: 'compressionLamports';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'pubkeyArray';
                        type: {
                            vec: 'publicKey';
                        };
                    },
                    {
                        name: 'message';
                        type: {
                            option: 'bytes';
                        };
                    },
                ];
            };
        },
        {
            name: 'CompressedCpiContext';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'setContext';
                        type: 'bool';
                    },
                    {
                        name: 'cpiContextAccountIndex';
                        type: 'u8';
                    },
                ];
            };
        },
    ];
    errors: [
        {
            code: 6000;
            name: 'SumCheckFailed';
            msg: 'Sum check failed';
        },
        {
            code: 6001;
            name: 'SignerCheckFailed';
            msg: 'Signer check failed';
        },
        {
            code: 6002;
            name: 'CpiSignerCheckFailed';
            msg: 'Cpi signer check failed';
        },
        {
            code: 6003;
            name: 'ComputeInputSumFailed';
            msg: 'Computing input sum failed.';
        },
        {
            code: 6004;
            name: 'ComputeOutputSumFailed';
            msg: 'Computing output sum failed.';
        },
        {
            code: 6005;
            name: 'ComputeRpcSumFailed';
            msg: 'Computing rpc sum failed.';
        },
        {
            code: 6006;
            name: 'InUtxosAlreadyAdded';
            msg: 'InUtxosAlreadyAdded';
        },
        {
            code: 6007;
            name: 'NumberOfLeavesMismatch';
            msg: 'NumberOfLeavesMismatch';
        },
        {
            code: 6008;
            name: 'MerkleTreePubkeysMismatch';
            msg: 'MerkleTreePubkeysMismatch';
        },
        {
            code: 6009;
            name: 'NullifierArrayPubkeysMismatch';
            msg: 'NullifierArrayPubkeysMismatch';
        },
        {
            code: 6010;
            name: 'InvalidNoopPubkey';
            msg: 'InvalidNoopPubkey';
        },
        {
            code: 6011;
            name: 'ProofVerificationFailed';
            msg: 'ProofVerificationFailed';
        },
        {
            code: 6012;
            name: 'CompressedAccountHashError';
            msg: 'CompressedAccountHashError';
        },
        {
            code: 6013;
            name: 'InvalidAddress';
            msg: 'InvalidAddress';
        },
        {
            code: 6014;
            name: 'InvalidAddressQueue';
            msg: 'InvalidAddressQueue';
        },
        {
            code: 6015;
            name: 'InvalidNullifierQueue';
            msg: 'InvalidNullifierQueue';
        },
        {
            code: 6016;
            name: 'DeriveAddressError';
            msg: 'DeriveAddressError';
        },
        {
            code: 6017;
            name: 'CompressSolTransferFailed';
            msg: 'CompressSolTransferFailed';
        },
        {
            code: 6018;
            name: 'CompressedSolPdaUndefinedForCompressSol';
            msg: 'CompressedSolPdaUndefinedForCompressSol';
        },
        {
            code: 6019;
            name: 'DeCompressLamportsUndefinedForCompressSol';
            msg: 'DeCompressLamportsUndefinedForCompressSol';
        },
        {
            code: 6020;
            name: 'CompressedSolPdaUndefinedForDecompressSol';
            msg: 'CompressedSolPdaUndefinedForDecompressSol';
        },
        {
            code: 6021;
            name: 'DeCompressLamportsUndefinedForDecompressSol';
            msg: 'DeCompressLamportsUndefinedForDecompressSol';
        },
        {
            code: 6022;
            name: 'DecompressRecipientUndefinedForDecompressSol';
            msg: 'DecompressRecipientUndefinedForDecompressSol';
        },
        {
            code: 6023;
            name: 'LengthMismatch';
            msg: 'LengthMismatch';
        },
        {
            code: 6024;
            name: 'DelegateUndefined';
            msg: 'DelegateUndefined while delegated amount is defined';
        },
        {
            code: 6025;
            name: 'CpiContextAccountUndefined';
            msg: 'CpiContextAccountUndefined';
        },
        {
            code: 6026;
            name: 'WriteAccessCheckFailed';
            msg: 'WriteAccessCheckFailed';
        },
        {
            code: 6027;
            name: 'InvokingProgramNotProvided';
            msg: 'InvokingProgramNotProvided';
        },
        {
            code: 6028;
            name: 'SignerSeedsNotProvided';
            msg: 'SignerSeedsNotProvided';
        },
        {
            code: 6029;
            name: 'AdditionOverflowForDecompressSol';
            msg: 'AdditionOverflowForDecompressSol';
        },
        {
            code: 6030;
            name: 'InsufficientLamportsForDecompressSol';
            msg: 'InsufficientLamportsForDecompressSol';
        },
        {
            code: 6031;
            name: 'CpiContextMissing';
            msg: 'InsufficientLamportsForCompressSol';
        },
        {
            code: 6032;
            name: 'InvalidMerkleTreeOwner';
            msg: 'InvalidMerkleTreeOwner';
        },
        {
            code: 6033;
            name: 'ProofIsNone';
            msg: 'ProofIsNone';
        },
        {
            code: 6034;
            name: 'InvalidMerkleTreeIndex';
            msg: 'InvalidMerkleTreeIndex';
        },
        {
            code: 6035;
            name: 'ProofIsSome';
            msg: 'ProofIsSome';
        },
    ];
};

export const IDL: LightCompressedPda = {
    version: '0.3.1',
    name: 'light_compressed_pda',
    constants: [
        {
            name: 'COMPRESSED_SOL_PDA_SEED',
            type: 'bytes',
            value: '[99, 111, 109, 112, 114, 101, 115, 115, 101, 100, 95, 115, 111, 108, 95, 112, 100, 97]',
        },
    ],
    instructions: [
        {
            name: 'initCpiContextAccount',
            accounts: [
                {
                    name: 'feePayer',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'cpiContextAccount',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'systemProgram',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'associatedMerkleTree',
                    isMut: false,
                    isSigner: false,
                },
            ],
            args: [],
        },
        {
            name: 'invoke',
            accounts: [
                {
                    name: 'feePayer',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'authority',
                    isMut: false,
                    isSigner: true,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'noopProgram',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'accountCompressionAuthority',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'accountCompressionProgram',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'compressedSolPda',
                    isMut: true,
                    isSigner: false,
                    isOptional: true,
                },
                {
                    name: 'compressionRecipient',
                    isMut: true,
                    isSigner: false,
                    isOptional: true,
                },
                {
                    name: 'systemProgram',
                    isMut: false,
                    isSigner: false,
                },
            ],
            args: [
                {
                    name: 'inputs',
                    type: 'bytes',
                },
            ],
        },
        {
            name: 'invokeCpi',
            accounts: [
                {
                    name: 'feePayer',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'authority',
                    isMut: false,
                    isSigner: true,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'noopProgram',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'accountCompressionAuthority',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'accountCompressionProgram',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'invokingProgram',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'compressedSolPda',
                    isMut: true,
                    isSigner: false,
                    isOptional: true,
                },
                {
                    name: 'compressionRecipient',
                    isMut: true,
                    isSigner: false,
                    isOptional: true,
                },
                {
                    name: 'systemProgram',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'cpiContextAccount',
                    isMut: true,
                    isSigner: false,
                    isOptional: true,
                },
            ],
            args: [
                {
                    name: 'inputs',
                    type: 'bytes',
                },
            ],
        },
    ],
    accounts: [
        {
            name: 'compressedSolPda',
            type: {
                kind: 'struct',
                fields: [],
            },
        },
        {
            name: 'cpiContextAccount',
            docs: [
                'collects invocations without proofs',
                'invocations are collected and processed when an invocation with a proof is received',
            ],
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'associatedMerkleTree',
                        type: 'publicKey',
                    },
                    {
                        name: 'context',
                        type: {
                            vec: {
                                defined: 'InstructionDataInvokeCpi',
                            },
                        },
                    },
                ],
            },
        },
    ],
    types: [
        {
            name: 'InstructionDataInvoke',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'proof',
                        type: {
                            option: {
                                defined: 'CompressedProof',
                            },
                        },
                    },
                    {
                        name: 'inputRootIndices',
                        type: {
                            vec: 'u16',
                        },
                    },
                    {
                        name: 'inputCompressedAccountsWithMerkleContext',
                        type: {
                            vec: {
                                defined:
                                    'PackedCompressedAccountWithMerkleContext',
                            },
                        },
                    },
                    {
                        name: 'outputCompressedAccounts',
                        type: {
                            vec: {
                                defined: 'CompressedAccount',
                            },
                        },
                    },
                    {
                        name: 'outputStateMerkleTreeAccountIndices',
                        docs: [
                            'The indices of the accounts in the output state merkle tree.',
                        ],
                        type: 'bytes',
                    },
                    {
                        name: 'relayFee',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'newAddressParams',
                        type: {
                            vec: {
                                defined: 'NewAddressParamsPacked',
                            },
                        },
                    },
                    {
                        name: 'compressionLamports',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'isCompress',
                        type: 'bool',
                    },
                ],
            },
        },
        {
            name: 'NewAddressParamsPacked',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'seed',
                        type: {
                            array: ['u8', 32],
                        },
                    },
                    {
                        name: 'addressQueueAccountIndex',
                        type: 'u8',
                    },
                    {
                        name: 'addressMerkleTreeAccountIndex',
                        type: 'u8',
                    },
                    {
                        name: 'addressMerkleTreeRootIndex',
                        type: 'u16',
                    },
                ],
            },
        },
        {
            name: 'NewAddressParams',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'seed',
                        type: {
                            array: ['u8', 32],
                        },
                    },
                    {
                        name: 'addressQueuePubkey',
                        type: 'publicKey',
                    },
                    {
                        name: 'addressMerkleTreePubkey',
                        type: 'publicKey',
                    },
                    {
                        name: 'addressMerkleTreeRootIndex',
                        type: 'u16',
                    },
                ],
            },
        },
        {
            name: 'CompressedProof',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'a',
                        type: {
                            array: ['u8', 32],
                        },
                    },
                    {
                        name: 'b',
                        type: {
                            array: ['u8', 64],
                        },
                    },
                    {
                        name: 'c',
                        type: {
                            array: ['u8', 32],
                        },
                    },
                ],
            },
        },
        {
            name: 'InstructionDataInvokeCpi',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'proof',
                        type: {
                            option: {
                                defined: 'CompressedProof',
                            },
                        },
                    },
                    {
                        name: 'newAddressParams',
                        type: {
                            vec: {
                                defined: 'NewAddressParamsPacked',
                            },
                        },
                    },
                    {
                        name: 'inputRootIndices',
                        type: {
                            vec: 'u16',
                        },
                    },
                    {
                        name: 'inputCompressedAccountsWithMerkleContext',
                        type: {
                            vec: {
                                defined:
                                    'PackedCompressedAccountWithMerkleContext',
                            },
                        },
                    },
                    {
                        name: 'outputCompressedAccounts',
                        type: {
                            vec: {
                                defined: 'CompressedAccount',
                            },
                        },
                    },
                    {
                        name: 'outputStateMerkleTreeAccountIndices',
                        docs: [
                            'The indices of the accounts in the output state merkle tree.',
                        ],
                        type: 'bytes',
                    },
                    {
                        name: 'relayFee',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'compressionLamports',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'isCompress',
                        type: 'bool',
                    },
                    {
                        name: 'signerSeeds',
                        type: {
                            vec: 'bytes',
                        },
                    },
                    {
                        name: 'cpiContext',
                        type: {
                            option: {
                                defined: 'CompressedCpiContext',
                            },
                        },
                    },
                ],
            },
        },
        {
            name: 'PackedCompressedAccountWithMerkleContext',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'compressedAccount',
                        type: {
                            defined: 'CompressedAccount',
                        },
                    },
                    {
                        name: 'merkleContext',
                        type: {
                            defined: 'PackedMerkleContext',
                        },
                    },
                ],
            },
        },
        {
            name: 'CompressedAccountWithMerkleContext',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'compressedAccount',
                        type: {
                            defined: 'CompressedAccount',
                        },
                    },
                    {
                        name: 'merkleContext',
                        type: {
                            defined: 'MerkleContext',
                        },
                    },
                ],
            },
        },
        {
            name: 'MerkleContext',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'merkleTreePubkey',
                        type: 'publicKey',
                    },
                    {
                        name: 'nullifierQueuePubkey',
                        type: 'publicKey',
                    },
                    {
                        name: 'leafIndex',
                        type: 'u32',
                    },
                ],
            },
        },
        {
            name: 'PackedMerkleContext',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'merkleTreePubkeyIndex',
                        type: 'u8',
                    },
                    {
                        name: 'nullifierQueuePubkeyIndex',
                        type: 'u8',
                    },
                    {
                        name: 'leafIndex',
                        type: 'u32',
                    },
                ],
            },
        },
        {
            name: 'CompressedAccount',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'owner',
                        type: 'publicKey',
                    },
                    {
                        name: 'lamports',
                        type: 'u64',
                    },
                    {
                        name: 'address',
                        type: {
                            option: {
                                array: ['u8', 32],
                            },
                        },
                    },
                    {
                        name: 'data',
                        type: {
                            option: {
                                defined: 'CompressedAccountData',
                            },
                        },
                    },
                ],
            },
        },
        {
            name: 'CompressedAccountData',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'discriminator',
                        type: {
                            array: ['u8', 8],
                        },
                    },
                    {
                        name: 'data',
                        type: 'bytes',
                    },
                    {
                        name: 'dataHash',
                        type: {
                            array: ['u8', 32],
                        },
                    },
                ],
            },
        },
        {
            name: 'PublicTransactionEvent',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'inputCompressedAccountHashes',
                        type: {
                            vec: {
                                array: ['u8', 32],
                            },
                        },
                    },
                    {
                        name: 'outputCompressedAccountHashes',
                        type: {
                            vec: {
                                array: ['u8', 32],
                            },
                        },
                    },
                    {
                        name: 'outputCompressedAccounts',
                        type: {
                            vec: {
                                defined: 'CompressedAccount',
                            },
                        },
                    },
                    {
                        name: 'outputStateMerkleTreeAccountIndices',
                        type: 'bytes',
                    },
                    {
                        name: 'outputLeafIndices',
                        type: {
                            vec: 'u32',
                        },
                    },
                    {
                        name: 'relayFee',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'isCompress',
                        type: 'bool',
                    },
                    {
                        name: 'compressionLamports',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'pubkeyArray',
                        type: {
                            vec: 'publicKey',
                        },
                    },
                    {
                        name: 'message',
                        type: {
                            option: 'bytes',
                        },
                    },
                ],
            },
        },
        {
            name: 'CompressedCpiContext',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'setContext',
                        type: 'bool',
                    },
                    {
                        name: 'cpiContextAccountIndex',
                        type: 'u8',
                    },
                ],
            },
        },
    ],
    errors: [
        {
            code: 6000,
            name: 'SumCheckFailed',
            msg: 'Sum check failed',
        },
        {
            code: 6001,
            name: 'SignerCheckFailed',
            msg: 'Signer check failed',
        },
        {
            code: 6002,
            name: 'CpiSignerCheckFailed',
            msg: 'Cpi signer check failed',
        },
        {
            code: 6003,
            name: 'ComputeInputSumFailed',
            msg: 'Computing input sum failed.',
        },
        {
            code: 6004,
            name: 'ComputeOutputSumFailed',
            msg: 'Computing output sum failed.',
        },
        {
            code: 6005,
            name: 'ComputeRpcSumFailed',
            msg: 'Computing rpc sum failed.',
        },
        {
            code: 6006,
            name: 'InUtxosAlreadyAdded',
            msg: 'InUtxosAlreadyAdded',
        },
        {
            code: 6007,
            name: 'NumberOfLeavesMismatch',
            msg: 'NumberOfLeavesMismatch',
        },
        {
            code: 6008,
            name: 'MerkleTreePubkeysMismatch',
            msg: 'MerkleTreePubkeysMismatch',
        },
        {
            code: 6009,
            name: 'NullifierArrayPubkeysMismatch',
            msg: 'NullifierArrayPubkeysMismatch',
        },
        {
            code: 6010,
            name: 'InvalidNoopPubkey',
            msg: 'InvalidNoopPubkey',
        },
        {
            code: 6011,
            name: 'ProofVerificationFailed',
            msg: 'ProofVerificationFailed',
        },
        {
            code: 6012,
            name: 'CompressedAccountHashError',
            msg: 'CompressedAccountHashError',
        },
        {
            code: 6013,
            name: 'InvalidAddress',
            msg: 'InvalidAddress',
        },
        {
            code: 6014,
            name: 'InvalidAddressQueue',
            msg: 'InvalidAddressQueue',
        },
        {
            code: 6015,
            name: 'InvalidNullifierQueue',
            msg: 'InvalidNullifierQueue',
        },
        {
            code: 6016,
            name: 'DeriveAddressError',
            msg: 'DeriveAddressError',
        },
        {
            code: 6017,
            name: 'CompressSolTransferFailed',
            msg: 'CompressSolTransferFailed',
        },
        {
            code: 6018,
            name: 'CompressedSolPdaUndefinedForCompressSol',
            msg: 'CompressedSolPdaUndefinedForCompressSol',
        },
        {
            code: 6019,
            name: 'DeCompressLamportsUndefinedForCompressSol',
            msg: 'DeCompressLamportsUndefinedForCompressSol',
        },
        {
            code: 6020,
            name: 'CompressedSolPdaUndefinedForDecompressSol',
            msg: 'CompressedSolPdaUndefinedForDecompressSol',
        },
        {
            code: 6021,
            name: 'DeCompressLamportsUndefinedForDecompressSol',
            msg: 'DeCompressLamportsUndefinedForDecompressSol',
        },
        {
            code: 6022,
            name: 'DecompressRecipientUndefinedForDecompressSol',
            msg: 'DecompressRecipientUndefinedForDecompressSol',
        },
        {
            code: 6023,
            name: 'LengthMismatch',
            msg: 'LengthMismatch',
        },
        {
            code: 6024,
            name: 'DelegateUndefined',
            msg: 'DelegateUndefined while delegated amount is defined',
        },
        {
            code: 6025,
            name: 'CpiContextAccountUndefined',
            msg: 'CpiContextAccountUndefined',
        },
        {
            code: 6026,
            name: 'WriteAccessCheckFailed',
            msg: 'WriteAccessCheckFailed',
        },
        {
            code: 6027,
            name: 'InvokingProgramNotProvided',
            msg: 'InvokingProgramNotProvided',
        },
        {
            code: 6028,
            name: 'SignerSeedsNotProvided',
            msg: 'SignerSeedsNotProvided',
        },
        {
            code: 6029,
            name: 'AdditionOverflowForDecompressSol',
            msg: 'AdditionOverflowForDecompressSol',
        },
        {
            code: 6030,
            name: 'InsufficientLamportsForDecompressSol',
            msg: 'InsufficientLamportsForDecompressSol',
        },
        {
            code: 6031,
            name: 'CpiContextMissing',
            msg: 'InsufficientLamportsForCompressSol',
        },
        {
            code: 6032,
            name: 'InvalidMerkleTreeOwner',
            msg: 'InvalidMerkleTreeOwner',
        },
        {
            code: 6033,
            name: 'ProofIsNone',
            msg: 'ProofIsNone',
        },
        {
            code: 6034,
            name: 'InvalidMerkleTreeIndex',
            msg: 'InvalidMerkleTreeIndex',
        },
        {
            code: 6035,
            name: 'ProofIsSome',
            msg: 'ProofIsSome',
        },
    ],
};
