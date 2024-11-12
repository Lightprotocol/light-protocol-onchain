export type AccountCompression = {
    version: '1.2.0';
    name: 'account_compression';
    constants: [
        {
            name: 'CPI_AUTHORITY_PDA_SEED';
            type: 'bytes';
            value: '[99, 112, 105, 95, 97, 117, 116, 104, 111, 114, 105, 116, 121]';
        },
        {
            name: 'GROUP_AUTHORITY_SEED';
            type: 'bytes';
            value: '[103, 114, 111, 117, 112, 95, 97, 117, 116, 104, 111, 114, 105, 116, 121]';
        },
        {
            name: 'STATE_MERKLE_TREE_HEIGHT';
            type: 'u64';
            value: '26';
        },
        {
            name: 'STATE_MERKLE_TREE_CHANGELOG';
            type: 'u64';
            value: '1400';
        },
        {
            name: 'STATE_MERKLE_TREE_ROOTS';
            type: 'u64';
            value: '2400';
        },
        {
            name: 'STATE_MERKLE_TREE_CANOPY_DEPTH';
            type: 'u64';
            value: '10';
        },
        {
            name: 'STATE_NULLIFIER_QUEUE_VALUES';
            type: 'u16';
            value: '28_807';
        },
        {
            name: 'STATE_NULLIFIER_QUEUE_SEQUENCE_THRESHOLD';
            type: 'u64';
            value: '2400';
        },
        {
            name: 'ADDRESS_MERKLE_TREE_HEIGHT';
            type: 'u64';
            value: '26';
        },
        {
            name: 'ADDRESS_MERKLE_TREE_CHANGELOG';
            type: 'u64';
            value: '1400';
        },
        {
            name: 'ADDRESS_MERKLE_TREE_ROOTS';
            type: 'u64';
            value: '2400';
        },
        {
            name: 'ADDRESS_MERKLE_TREE_CANOPY_DEPTH';
            type: 'u64';
            value: '10';
        },
        {
            name: 'ADDRESS_MERKLE_TREE_INDEXED_CHANGELOG';
            type: 'u64';
            value: '1400';
        },
        {
            name: 'ADDRESS_QUEUE_VALUES';
            type: 'u16';
            value: '28_807';
        },
        {
            name: 'ADDRESS_QUEUE_SEQUENCE_THRESHOLD';
            type: 'u64';
            value: '2400';
        },
        {
            name: 'NOOP_PUBKEY';
            type: {
                array: ['u8', 32];
            };
            value: '[11 , 188 , 15 , 192 , 187 , 71 , 202 , 47 , 116 , 196 , 17 , 46 , 148 , 171 , 19 , 207 , 163 , 198 , 52 , 229 , 220 , 23 , 234 , 203 , 3 , 205 , 26 , 35 , 205 , 126 , 120 , 124 ,]';
        },
        {
            name: 'TEST_DEFAULT_BATCH_SIZE';
            type: 'u64';
            value: '50';
        },
        {
            name: 'TEST_DEFAULT_ZKP_BATCH_SIZE';
            type: 'u64';
            value: '10';
        },
        {
            name: 'DEFAULT_BATCH_SIZE';
            type: 'u64';
            value: '50000';
        },
        {
            name: 'DEFAULT_ZKP_BATCH_SIZE';
            type: 'u64';
            value: '500';
        },
    ];
    instructions: [
        {
            name: 'initializeBatchedStateMerkleTree';
            accounts: [
                {
                    name: 'authority';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'merkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'queue';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                    isOptional: true;
                },
            ];
            args: [
                {
                    name: 'params';
                    type: {
                        defined: 'InitStateTreeAccountsInstructionData';
                    };
                },
            ];
        },
        {
            name: 'initializeAddressMerkleTreeAndQueue';
            accounts: [
                {
                    name: 'authority';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'merkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'queue';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                    isOptional: true;
                },
            ];
            args: [
                {
                    name: 'index';
                    type: 'u64';
                },
                {
                    name: 'programOwner';
                    type: {
                        option: 'publicKey';
                    };
                },
                {
                    name: 'forester';
                    type: {
                        option: 'publicKey';
                    };
                },
                {
                    name: 'addressMerkleTreeConfig';
                    type: {
                        defined: 'AddressMerkleTreeConfig';
                    };
                },
                {
                    name: 'addressQueueConfig';
                    type: {
                        defined: 'AddressQueueConfig';
                    };
                },
            ];
        },
        {
            name: 'insertAddresses';
            accounts: [
                {
                    name: 'feePayer';
                    isMut: true;
                    isSigner: true;
                    docs: ['Fee payer pays rollover fee.'];
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
                    name: 'addresses';
                    type: {
                        vec: {
                            array: ['u8', 32];
                        };
                    };
                },
            ];
        },
        {
            name: 'updateAddressMerkleTree';
            docs: ['Updates the address Merkle tree with a new address.'];
            accounts: [
                {
                    name: 'authority';
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                    isOptional: true;
                },
                {
                    name: 'queue';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'merkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'logWrapper';
                    isMut: false;
                    isSigner: false;
                },
            ];
            args: [
                {
                    name: 'changelogIndex';
                    type: 'u16';
                },
                {
                    name: 'indexedChangelogIndex';
                    type: 'u16';
                },
                {
                    name: 'value';
                    type: 'u16';
                },
                {
                    name: 'lowAddressIndex';
                    type: 'u64';
                },
                {
                    name: 'lowAddressValue';
                    type: {
                        array: ['u8', 32];
                    };
                },
                {
                    name: 'lowAddressNextIndex';
                    type: 'u64';
                },
                {
                    name: 'lowAddressNextValue';
                    type: {
                        array: ['u8', 32];
                    };
                },
                {
                    name: 'lowAddressProof';
                    type: {
                        array: [
                            {
                                array: ['u8', 32];
                            },
                            16,
                        ];
                    };
                },
            ];
        },
        {
            name: 'rolloverAddressMerkleTreeAndQueue';
            accounts: [
                {
                    name: 'feePayer';
                    isMut: true;
                    isSigner: true;
                    docs: [
                        'Signer used to receive rollover accounts rentexemption reimbursement.',
                    ];
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
                    isOptional: true;
                },
                {
                    name: 'newAddressMerkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'newQueue';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'oldAddressMerkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'oldQueue';
                    isMut: true;
                    isSigner: false;
                },
            ];
            args: [];
        },
        {
            name: 'initializeGroupAuthority';
            docs: [
                'initialize group (a group can be used to give multiple programs access',
                'to the same Merkle trees by registering the programs to the group)',
            ];
            accounts: [
                {
                    name: 'authority';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'seed';
                    isMut: false;
                    isSigner: true;
                    docs: [
                        'Seed public key used to derive the group authority.',
                    ];
                },
                {
                    name: 'groupAuthority';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'systemProgram';
                    isMut: false;
                    isSigner: false;
                },
            ];
            args: [
                {
                    name: 'authority';
                    type: 'publicKey';
                },
            ];
        },
        {
            name: 'updateGroupAuthority';
            accounts: [
                {
                    name: 'authority';
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: 'groupAuthority';
                    isMut: true;
                    isSigner: false;
                },
            ];
            args: [
                {
                    name: 'authority';
                    type: 'publicKey';
                },
            ];
        },
        {
            name: 'registerProgramToGroup';
            accounts: [
                {
                    name: 'authority';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'programToBeRegistered';
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'groupAuthorityPda';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'systemProgram';
                    isMut: false;
                    isSigner: false;
                },
            ];
            args: [];
        },
        {
            name: 'deregisterProgram';
            accounts: [
                {
                    name: 'authority';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'groupAuthorityPda';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'closeRecipient';
                    isMut: true;
                    isSigner: false;
                },
            ];
            args: [];
        },
        {
            name: 'initializeStateMerkleTreeAndNullifierQueue';
            docs: [
                'Initializes a new Merkle tree from config bytes.',
                'Index is an optional identifier and not checked by the program.',
            ];
            accounts: [
                {
                    name: 'authority';
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: 'merkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'nullifierQueue';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                    isOptional: true;
                },
            ];
            args: [
                {
                    name: 'index';
                    type: 'u64';
                },
                {
                    name: 'programOwner';
                    type: {
                        option: 'publicKey';
                    };
                },
                {
                    name: 'forester';
                    type: {
                        option: 'publicKey';
                    };
                },
                {
                    name: 'stateMerkleTreeConfig';
                    type: {
                        defined: 'StateMerkleTreeConfig';
                    };
                },
                {
                    name: 'nullifierQueueConfig';
                    type: {
                        defined: 'NullifierQueueConfig';
                    };
                },
                {
                    name: 'additionalBytes';
                    type: 'u64';
                },
            ];
        },
        {
            name: 'appendLeavesToMerkleTrees';
            accounts: [
                {
                    name: 'feePayer';
                    isMut: true;
                    isSigner: true;
                    docs: ['Fee payer pays rollover fee.'];
                },
                {
                    name: 'authority';
                    isMut: false;
                    isSigner: true;
                    docs: [
                        'Checked whether instruction is accessed by a registered program or owner = authority.',
                    ];
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                    isOptional: true;
                    docs: [
                        'Some assumes that the Merkle trees are accessed by a registered program.',
                        'None assumes that the Merkle trees are accessed by its owner.',
                    ];
                },
                {
                    name: 'systemProgram';
                    isMut: false;
                    isSigner: false;
                },
            ];
            args: [
                {
                    name: 'leaves';
                    type: {
                        vec: {
                            defined: '(u8,[u8;32])';
                        };
                    };
                },
            ];
        },
        {
            name: 'nullifyLeaves';
            accounts: [
                {
                    name: 'authority';
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                    isOptional: true;
                },
                {
                    name: 'logWrapper';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'merkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'nullifierQueue';
                    isMut: true;
                    isSigner: false;
                },
            ];
            args: [
                {
                    name: 'changeLogIndices';
                    type: {
                        vec: 'u64';
                    };
                },
                {
                    name: 'leavesQueueIndices';
                    type: {
                        vec: 'u16';
                    };
                },
                {
                    name: 'leafIndices';
                    type: {
                        vec: 'u64';
                    };
                },
                {
                    name: 'proofs';
                    type: {
                        vec: {
                            vec: {
                                array: ['u8', 32];
                            };
                        };
                    };
                },
            ];
        },
        {
            name: 'insertIntoNullifierQueues';
            accounts: [
                {
                    name: 'feePayer';
                    isMut: true;
                    isSigner: true;
                    docs: ['Fee payer pays rollover fee.'];
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
                    name: 'nullifiers';
                    type: {
                        vec: {
                            array: ['u8', 32];
                        };
                    };
                },
                {
                    name: 'leafIndices';
                    type: {
                        vec: 'u32';
                    };
                },
                {
                    name: 'txHash';
                    type: {
                        option: {
                            array: ['u8', 32];
                        };
                    };
                },
                {
                    name: 'checkProofByIndex';
                    type: {
                        option: {
                            vec: 'bool';
                        };
                    };
                },
            ];
        },
        {
            name: 'rolloverStateMerkleTreeAndNullifierQueue';
            accounts: [
                {
                    name: 'feePayer';
                    isMut: true;
                    isSigner: true;
                    docs: [
                        'Signer used to receive rollover accounts rentexemption reimbursement.',
                    ];
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
                    isOptional: true;
                },
                {
                    name: 'newStateMerkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'newNullifierQueue';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'oldStateMerkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'oldNullifierQueue';
                    isMut: true;
                    isSigner: false;
                },
            ];
            args: [];
        },
        {
            name: 'batchNullify';
            accounts: [
                {
                    name: 'authority';
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                    isOptional: true;
                },
                {
                    name: 'logWrapper';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'merkleTree';
                    isMut: true;
                    isSigner: false;
                },
            ];
            args: [
                {
                    name: 'data';
                    type: 'bytes';
                },
            ];
        },
        {
            name: 'batchAppend';
            accounts: [
                {
                    name: 'authority';
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: 'registeredProgramPda';
                    isMut: false;
                    isSigner: false;
                    isOptional: true;
                },
                {
                    name: 'logWrapper';
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: 'merkleTree';
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: 'outputQueue';
                    isMut: true;
                    isSigner: false;
                },
            ];
            args: [
                {
                    name: 'data';
                    type: 'bytes';
                },
            ];
        },
    ];
    accounts: [
        {
            name: 'registeredProgram';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'registeredProgramId';
                        type: 'publicKey';
                    },
                    {
                        name: 'groupAuthorityPda';
                        type: 'publicKey';
                    },
                ];
            };
        },
        {
            name: 'accessMetadata';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'owner';
                        docs: ['Owner of the Merkle tree.'];
                        type: 'publicKey';
                    },
                    {
                        name: 'programOwner';
                        docs: [
                            'Program owner of the Merkle tree. This will be used for program owned Merkle trees.',
                        ];
                        type: 'publicKey';
                    },
                    {
                        name: 'forester';
                        docs: [
                            'Optional privileged forester pubkey, can be set for custom Merkle trees',
                            'without a network fee. Merkle trees without network fees are not',
                            'forested by light foresters. The variable is not used in the account',
                            'compression program but the registry program. The registry program',
                            'implements access control to prevent contention during forester. The',
                            'forester pubkey specified in this struct can bypass contention checks.',
                        ];
                        type: 'publicKey';
                    },
                ];
            };
        },
        {
            name: 'addressMerkleTreeAccount';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'metadata';
                        type: {
                            defined: 'MerkleTreeMetadata';
                        };
                    },
                ];
            };
        },
        {
            name: 'batchedMerkleTreeMetadata';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'accessMetadata';
                        type: {
                            defined: 'AccessMetadata';
                        };
                    },
                    {
                        name: 'rolloverMetadata';
                        type: {
                            defined: 'RolloverMetadata';
                        };
                    },
                    {
                        name: 'associatedOutputQueue';
                        type: 'publicKey';
                    },
                    {
                        name: 'nextMerkleTree';
                        type: 'publicKey';
                    },
                    {
                        name: 'treeType';
                        type: 'u64';
                    },
                ];
            };
        },
        {
            name: 'batchedMerkleTreeAccount';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'metadata';
                        type: {
                            defined: 'MerkleTreeMetadata';
                        };
                    },
                    {
                        name: 'sequenceNumber';
                        type: 'u64';
                    },
                    {
                        name: 'treeType';
                        type: 'u64';
                    },
                    {
                        name: 'nextIndex';
                        type: 'u64';
                    },
                    {
                        name: 'height';
                        type: 'u32';
                    },
                    {
                        name: 'rootHistoryCapacity';
                        type: 'u32';
                    },
                    {
                        name: 'queue';
                        type: {
                            defined: 'BatchedQueue';
                        };
                    },
                ];
            };
        },
        {
            name: 'batchedQueueAccount';
            docs: [
                'Memory layout:',
                '1. QueueMetadata',
                '2. num_batches: u64',
                '3. hash_chain hash bounded vec',
                '3. for num_batches every 33 bytes is a bloom filter',
                '3. (output queue) rest of account is bounded vec',
                '',
                'One Batch account contains multiple batches.',
            ];
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'metadata';
                        type: {
                            defined: 'QueueMetadata';
                        };
                    },
                    {
                        name: 'queue';
                        type: {
                            defined: 'BatchedQueue';
                        };
                    },
                    {
                        name: 'nextIndex';
                        docs: [
                            'Output queue requires next index to derive compressed account hashes.',
                            'next_index in queue is ahead or equal to next index in the associated',
                            'batched Merkle tree account.',
                        ];
                        type: 'u64';
                    },
                ];
            };
        },
        {
            name: 'batchedQueue';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'numBatches';
                        type: 'u64';
                    },
                    {
                        name: 'batchSize';
                        type: 'u64';
                    },
                    {
                        name: 'zkpBatchSize';
                        type: 'u64';
                    },
                    {
                        name: 'currentlyProcessingBatchIndex';
                        type: 'u64';
                    },
                    {
                        name: 'nextFullBatchIndex';
                        type: 'u64';
                    },
                    {
                        name: 'bloomFilterCapacity';
                        type: 'u64';
                    },
                ];
            };
        },
        {
            name: 'groupAuthority';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'authority';
                        type: 'publicKey';
                    },
                    {
                        name: 'seed';
                        type: 'publicKey';
                    },
                ];
            };
        },
        {
            name: 'merkleTreeMetadata';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'accessMetadata';
                        type: {
                            defined: 'AccessMetadata';
                        };
                    },
                    {
                        name: 'rolloverMetadata';
                        type: {
                            defined: 'RolloverMetadata';
                        };
                    },
                    {
                        name: 'associatedQueue';
                        type: 'publicKey';
                    },
                    {
                        name: 'nextMerkleTree';
                        type: 'publicKey';
                    },
                ];
            };
        },
        {
            name: 'stateMerkleTreeAccount';
            docs: [
                'Concurrent state Merkle tree used for public compressed transactions.',
            ];
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'metadata';
                        type: {
                            defined: 'MerkleTreeMetadata';
                        };
                    },
                ];
            };
        },
        {
            name: 'queueMetadata';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'accessMetadata';
                        type: {
                            defined: 'AccessMetadata';
                        };
                    },
                    {
                        name: 'rolloverMetadata';
                        type: {
                            defined: 'RolloverMetadata';
                        };
                    },
                    {
                        name: 'associatedMerkleTree';
                        type: 'publicKey';
                    },
                    {
                        name: 'nextQueue';
                        type: 'publicKey';
                    },
                    {
                        name: 'queueType';
                        type: 'u64';
                    },
                ];
            };
        },
        {
            name: 'queueAccount';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'metadata';
                        type: {
                            defined: 'QueueMetadata';
                        };
                    },
                ];
            };
        },
        {
            name: 'rolloverMetadata';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'index';
                        docs: ['Unique index.'];
                        type: 'u64';
                    },
                    {
                        name: 'rolloverFee';
                        docs: [
                            'This fee is used for rent for the next account.',
                            'It accumulates in the account so that once the corresponding Merkle tree account is full it can be rolled over',
                        ];
                        type: 'u64';
                    },
                    {
                        name: 'rolloverThreshold';
                        docs: [
                            'The threshold in percentage points when the account should be rolled over (95 corresponds to 95% filled).',
                        ];
                        type: 'u64';
                    },
                    {
                        name: 'networkFee';
                        docs: ['Tip for maintaining the account.'];
                        type: 'u64';
                    },
                    {
                        name: 'rolledoverSlot';
                        docs: [
                            'The slot when the account was rolled over, a rolled over account should not be written to.',
                        ];
                        type: 'u64';
                    },
                    {
                        name: 'closeThreshold';
                        docs: [
                            'If current slot is greater than rolledover_slot + close_threshold and',
                            "the account is empty it can be closed. No 'close' functionality has been",
                            'implemented yet.',
                        ];
                        type: 'u64';
                    },
                    {
                        name: 'additionalBytes';
                        docs: [
                            'Placeholder for bytes of additional accounts which are tied to the',
                            'Merkle trees operation and need to be rolled over as well.',
                        ];
                        type: 'u64';
                    },
                ];
            };
        },
    ];
    types: [
        {
            name: 'ZeroOutLeafIndex';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'treeIndex';
                        type: 'u8';
                    },
                    {
                        name: 'batchIndex';
                        type: 'u8';
                    },
                    {
                        name: 'leafIndex';
                        type: 'u16';
                    },
                ];
            };
        },
        {
            name: 'AddressMerkleTreeConfig';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'height';
                        type: 'u32';
                    },
                    {
                        name: 'changelogSize';
                        type: 'u64';
                    },
                    {
                        name: 'rootsSize';
                        type: 'u64';
                    },
                    {
                        name: 'canopyDepth';
                        type: 'u64';
                    },
                    {
                        name: 'addressChangelogSize';
                        type: 'u64';
                    },
                    {
                        name: 'networkFee';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'rolloverThreshold';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'closeThreshold';
                        type: {
                            option: 'u64';
                        };
                    },
                ];
            };
        },
        {
            name: 'StateMerkleTreeConfig';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'height';
                        type: 'u32';
                    },
                    {
                        name: 'changelogSize';
                        type: 'u64';
                    },
                    {
                        name: 'rootsSize';
                        type: 'u64';
                    },
                    {
                        name: 'canopyDepth';
                        type: 'u64';
                    },
                    {
                        name: 'networkFee';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'rolloverThreshold';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'closeThreshold';
                        type: {
                            option: 'u64';
                        };
                    },
                ];
            };
        },
        {
            name: 'NullifierQueueConfig';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'capacity';
                        type: 'u16';
                    },
                    {
                        name: 'sequenceThreshold';
                        type: 'u64';
                    },
                    {
                        name: 'networkFee';
                        type: {
                            option: 'u64';
                        };
                    },
                ];
            };
        },
        {
            name: 'InitStateTreeAccountsInstructionData';
            type: {
                kind: 'struct';
                fields: [
                    {
                        name: 'index';
                        type: 'u64';
                    },
                    {
                        name: 'programOwner';
                        type: {
                            option: 'publicKey';
                        };
                    },
                    {
                        name: 'forester';
                        type: {
                            option: 'publicKey';
                        };
                    },
                    {
                        name: 'additionalBytes';
                        type: 'u64';
                    },
                    {
                        name: 'inputQueueBatchSize';
                        type: 'u64';
                    },
                    {
                        name: 'outputQueueBatchSize';
                        type: 'u64';
                    },
                    {
                        name: 'inputQueueZkpBatchSize';
                        type: 'u64';
                    },
                    {
                        name: 'outputQueueZkpBatchSize';
                        type: 'u64';
                    },
                    {
                        name: 'bloomFilterNumIters';
                        type: 'u64';
                    },
                    {
                        name: 'bloomFilterCapacity';
                        type: 'u64';
                    },
                    {
                        name: 'rootHistoryCapacity';
                        type: 'u32';
                    },
                    {
                        name: 'networkFee';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'rolloverThreshold';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'closeThreshold';
                        type: {
                            option: 'u64';
                        };
                    },
                    {
                        name: 'inputQueueNumBatches';
                        type: 'u64';
                    },
                    {
                        name: 'outputQueueNumBatches';
                        type: 'u64';
                    },
                    {
                        name: 'height';
                        type: 'u32';
                    },
                ];
            };
        },
        {
            name: 'BatchState';
            type: {
                kind: 'enum';
                variants: [
                    {
                        name: 'CanBeFilled';
                    },
                    {
                        name: 'Inserted';
                    },
                    {
                        name: 'ReadyToUpdateTree';
                    },
                ];
            };
        },
        {
            name: 'TreeType';
            type: {
                kind: 'enum';
                variants: [
                    {
                        name: 'State';
                    },
                    {
                        name: 'Address';
                    },
                    {
                        name: 'BatchedState';
                    },
                    {
                        name: 'BatchedAddress';
                    },
                ];
            };
        },
        {
            name: 'QueueType';
            type: {
                kind: 'enum';
                variants: [
                    {
                        name: 'NullifierQueue';
                    },
                    {
                        name: 'AddressQueue';
                    },
                    {
                        name: 'Input';
                    },
                    {
                        name: 'Address';
                    },
                    {
                        name: 'Output';
                    },
                ];
            };
        },
        {
            name: 'AddressQueueConfig';
            type: {
                kind: 'alias';
                value: {
                    defined: 'NullifierQueueConfig';
                };
            };
        },
    ];
    errors: [
        {
            code: 6000;
            name: 'IntegerOverflow';
            msg: 'Integer overflow';
        },
        {
            code: 6001;
            name: 'InvalidAuthority';
            msg: 'InvalidAuthority';
        },
        {
            code: 6002;
            name: 'NumberOfLeavesMismatch';
            msg: 'Leaves <> remaining accounts mismatch. The number of remaining accounts must match the number of leaves.';
        },
        {
            code: 6003;
            name: 'InvalidNoopPubkey';
            msg: 'Provided noop program public key is invalid';
        },
        {
            code: 6004;
            name: 'NumberOfChangeLogIndicesMismatch';
            msg: 'Number of change log indices mismatch';
        },
        {
            code: 6005;
            name: 'NumberOfIndicesMismatch';
            msg: 'Number of indices mismatch';
        },
        {
            code: 6006;
            name: 'NumberOfProofsMismatch';
            msg: 'NumberOfProofsMismatch';
        },
        {
            code: 6007;
            name: 'InvalidMerkleProof';
            msg: 'InvalidMerkleProof';
        },
        {
            code: 6008;
            name: 'LeafNotFound';
            msg: 'Could not find the leaf in the queue';
        },
        {
            code: 6009;
            name: 'MerkleTreeAndQueueNotAssociated';
            msg: 'MerkleTreeAndQueueNotAssociated';
        },
        {
            code: 6010;
            name: 'MerkleTreeAlreadyRolledOver';
            msg: 'MerkleTreeAlreadyRolledOver';
        },
        {
            code: 6011;
            name: 'NotReadyForRollover';
            msg: 'NotReadyForRollover';
        },
        {
            code: 6012;
            name: 'RolloverNotConfigured';
            msg: 'RolloverNotConfigured';
        },
        {
            code: 6013;
            name: 'NotAllLeavesProcessed';
            msg: 'NotAllLeavesProcessed';
        },
        {
            code: 6014;
            name: 'InvalidQueueType';
            msg: 'InvalidQueueType';
        },
        {
            code: 6015;
            name: 'InputElementsEmpty';
            msg: 'InputElementsEmpty';
        },
        {
            code: 6016;
            name: 'NoLeavesForMerkleTree';
            msg: 'NoLeavesForMerkleTree';
        },
        {
            code: 6017;
            name: 'InvalidAccountSize';
            msg: 'InvalidAccountSize';
        },
        {
            code: 6018;
            name: 'InsufficientRolloverFee';
            msg: 'InsufficientRolloverFee';
        },
        {
            code: 6019;
            name: 'UnsupportedHeight';
            msg: 'Unsupported Merkle tree height';
        },
        {
            code: 6020;
            name: 'UnsupportedCanopyDepth';
            msg: 'Unsupported canopy depth';
        },
        {
            code: 6021;
            name: 'InvalidSequenceThreshold';
            msg: 'Invalid sequence threshold';
        },
        {
            code: 6022;
            name: 'UnsupportedCloseThreshold';
            msg: 'Unsupported close threshold';
        },
        {
            code: 6023;
            name: 'InvalidAccountBalance';
            msg: 'InvalidAccountBalance';
        },
        {
            code: 6024;
            name: 'UnsupportedAdditionalBytes';
        },
        {
            code: 6025;
            name: 'InvalidGroup';
        },
        {
            code: 6026;
            name: 'ProofLengthMismatch';
        },
        {
            code: 6027;
            name: 'InvalidCommitmentLength';
            msg: 'Invalid commitment length';
        },
        {
            code: 6028;
            name: 'BloomFilterFull';
            msg: 'BloomFilterFull';
        },
        {
            code: 6029;
            name: 'BatchInsertFailed';
            msg: 'BatchInsertFailed';
        },
        {
            code: 6030;
            name: 'BatchNotReady';
            msg: 'BatchNotReady';
        },
        {
            code: 6031;
            name: 'SizeMismatch';
        },
        {
            code: 6032;
            name: 'BatchAlreadyInserted';
        },
        {
            code: 6033;
            name: 'InvalidBloomFilterCapacity';
        },
        {
            code: 6034;
            name: 'InvalidCircuitBatchSize';
        },
        {
            code: 6035;
            name: 'InvalidDiscriminator';
        },
        {
            code: 6036;
            name: 'BatchSizeNotDivisibleByZkpBatchSize';
            msg: 'batch_size is not divisible by zkp_batch_size';
        },
        {
            code: 6037;
            name: 'InclusionProofByIndexFailed';
        },
        {
            code: 6038;
            name: 'TxHashUndefined';
        },
        {
            code: 6039;
            name: 'InputDeserializationFailed';
        },
        {
            code: 6040;
            name: 'InvalidBatch';
        },
        {
            code: 6041;
            name: 'LeafIndexNotInBatch';
        },
        {
            code: 6042;
            name: 'UnsupportedParameters';
        },
    ];
};

export const IDL: AccountCompression = {
    version: '1.2.0',
    name: 'account_compression',
    constants: [
        {
            name: 'CPI_AUTHORITY_PDA_SEED',
            type: 'bytes',
            value: '[99, 112, 105, 95, 97, 117, 116, 104, 111, 114, 105, 116, 121]',
        },
        {
            name: 'GROUP_AUTHORITY_SEED',
            type: 'bytes',
            value: '[103, 114, 111, 117, 112, 95, 97, 117, 116, 104, 111, 114, 105, 116, 121]',
        },
        {
            name: 'STATE_MERKLE_TREE_HEIGHT',
            type: 'u64',
            value: '26',
        },
        {
            name: 'STATE_MERKLE_TREE_CHANGELOG',
            type: 'u64',
            value: '1400',
        },
        {
            name: 'STATE_MERKLE_TREE_ROOTS',
            type: 'u64',
            value: '2400',
        },
        {
            name: 'STATE_MERKLE_TREE_CANOPY_DEPTH',
            type: 'u64',
            value: '10',
        },
        {
            name: 'STATE_NULLIFIER_QUEUE_VALUES',
            type: 'u16',
            value: '28_807',
        },
        {
            name: 'STATE_NULLIFIER_QUEUE_SEQUENCE_THRESHOLD',
            type: 'u64',
            value: '2400',
        },
        {
            name: 'ADDRESS_MERKLE_TREE_HEIGHT',
            type: 'u64',
            value: '26',
        },
        {
            name: 'ADDRESS_MERKLE_TREE_CHANGELOG',
            type: 'u64',
            value: '1400',
        },
        {
            name: 'ADDRESS_MERKLE_TREE_ROOTS',
            type: 'u64',
            value: '2400',
        },
        {
            name: 'ADDRESS_MERKLE_TREE_CANOPY_DEPTH',
            type: 'u64',
            value: '10',
        },
        {
            name: 'ADDRESS_MERKLE_TREE_INDEXED_CHANGELOG',
            type: 'u64',
            value: '1400',
        },
        {
            name: 'ADDRESS_QUEUE_VALUES',
            type: 'u16',
            value: '28_807',
        },
        {
            name: 'ADDRESS_QUEUE_SEQUENCE_THRESHOLD',
            type: 'u64',
            value: '2400',
        },
        {
            name: 'NOOP_PUBKEY',
            type: {
                array: ['u8', 32],
            },
            value: '[11 , 188 , 15 , 192 , 187 , 71 , 202 , 47 , 116 , 196 , 17 , 46 , 148 , 171 , 19 , 207 , 163 , 198 , 52 , 229 , 220 , 23 , 234 , 203 , 3 , 205 , 26 , 35 , 205 , 126 , 120 , 124 ,]',
        },
        {
            name: 'TEST_DEFAULT_BATCH_SIZE',
            type: 'u64',
            value: '50',
        },
        {
            name: 'TEST_DEFAULT_ZKP_BATCH_SIZE',
            type: 'u64',
            value: '10',
        },
        {
            name: 'DEFAULT_BATCH_SIZE',
            type: 'u64',
            value: '50000',
        },
        {
            name: 'DEFAULT_ZKP_BATCH_SIZE',
            type: 'u64',
            value: '500',
        },
    ],
    instructions: [
        {
            name: 'initializeBatchedStateMerkleTree',
            accounts: [
                {
                    name: 'authority',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'merkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'queue',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                    isOptional: true,
                },
            ],
            args: [
                {
                    name: 'params',
                    type: {
                        defined: 'InitStateTreeAccountsInstructionData',
                    },
                },
            ],
        },
        {
            name: 'initializeAddressMerkleTreeAndQueue',
            accounts: [
                {
                    name: 'authority',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'merkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'queue',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                    isOptional: true,
                },
            ],
            args: [
                {
                    name: 'index',
                    type: 'u64',
                },
                {
                    name: 'programOwner',
                    type: {
                        option: 'publicKey',
                    },
                },
                {
                    name: 'forester',
                    type: {
                        option: 'publicKey',
                    },
                },
                {
                    name: 'addressMerkleTreeConfig',
                    type: {
                        defined: 'AddressMerkleTreeConfig',
                    },
                },
                {
                    name: 'addressQueueConfig',
                    type: {
                        defined: 'AddressQueueConfig',
                    },
                },
            ],
        },
        {
            name: 'insertAddresses',
            accounts: [
                {
                    name: 'feePayer',
                    isMut: true,
                    isSigner: true,
                    docs: ['Fee payer pays rollover fee.'],
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
                    name: 'addresses',
                    type: {
                        vec: {
                            array: ['u8', 32],
                        },
                    },
                },
            ],
        },
        {
            name: 'updateAddressMerkleTree',
            docs: ['Updates the address Merkle tree with a new address.'],
            accounts: [
                {
                    name: 'authority',
                    isMut: false,
                    isSigner: true,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                    isOptional: true,
                },
                {
                    name: 'queue',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'merkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'logWrapper',
                    isMut: false,
                    isSigner: false,
                },
            ],
            args: [
                {
                    name: 'changelogIndex',
                    type: 'u16',
                },
                {
                    name: 'indexedChangelogIndex',
                    type: 'u16',
                },
                {
                    name: 'value',
                    type: 'u16',
                },
                {
                    name: 'lowAddressIndex',
                    type: 'u64',
                },
                {
                    name: 'lowAddressValue',
                    type: {
                        array: ['u8', 32],
                    },
                },
                {
                    name: 'lowAddressNextIndex',
                    type: 'u64',
                },
                {
                    name: 'lowAddressNextValue',
                    type: {
                        array: ['u8', 32],
                    },
                },
                {
                    name: 'lowAddressProof',
                    type: {
                        array: [
                            {
                                array: ['u8', 32],
                            },
                            16,
                        ],
                    },
                },
            ],
        },
        {
            name: 'rolloverAddressMerkleTreeAndQueue',
            accounts: [
                {
                    name: 'feePayer',
                    isMut: true,
                    isSigner: true,
                    docs: [
                        'Signer used to receive rollover accounts rentexemption reimbursement.',
                    ],
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
                    isOptional: true,
                },
                {
                    name: 'newAddressMerkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'newQueue',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'oldAddressMerkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'oldQueue',
                    isMut: true,
                    isSigner: false,
                },
            ],
            args: [],
        },
        {
            name: 'initializeGroupAuthority',
            docs: [
                'initialize group (a group can be used to give multiple programs access',
                'to the same Merkle trees by registering the programs to the group)',
            ],
            accounts: [
                {
                    name: 'authority',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'seed',
                    isMut: false,
                    isSigner: true,
                    docs: [
                        'Seed public key used to derive the group authority.',
                    ],
                },
                {
                    name: 'groupAuthority',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'systemProgram',
                    isMut: false,
                    isSigner: false,
                },
            ],
            args: [
                {
                    name: 'authority',
                    type: 'publicKey',
                },
            ],
        },
        {
            name: 'updateGroupAuthority',
            accounts: [
                {
                    name: 'authority',
                    isMut: false,
                    isSigner: true,
                },
                {
                    name: 'groupAuthority',
                    isMut: true,
                    isSigner: false,
                },
            ],
            args: [
                {
                    name: 'authority',
                    type: 'publicKey',
                },
            ],
        },
        {
            name: 'registerProgramToGroup',
            accounts: [
                {
                    name: 'authority',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'programToBeRegistered',
                    isMut: false,
                    isSigner: true,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'groupAuthorityPda',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'systemProgram',
                    isMut: false,
                    isSigner: false,
                },
            ],
            args: [],
        },
        {
            name: 'deregisterProgram',
            accounts: [
                {
                    name: 'authority',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'groupAuthorityPda',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'closeRecipient',
                    isMut: true,
                    isSigner: false,
                },
            ],
            args: [],
        },
        {
            name: 'initializeStateMerkleTreeAndNullifierQueue',
            docs: [
                'Initializes a new Merkle tree from config bytes.',
                'Index is an optional identifier and not checked by the program.',
            ],
            accounts: [
                {
                    name: 'authority',
                    isMut: true,
                    isSigner: true,
                },
                {
                    name: 'merkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'nullifierQueue',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                    isOptional: true,
                },
            ],
            args: [
                {
                    name: 'index',
                    type: 'u64',
                },
                {
                    name: 'programOwner',
                    type: {
                        option: 'publicKey',
                    },
                },
                {
                    name: 'forester',
                    type: {
                        option: 'publicKey',
                    },
                },
                {
                    name: 'stateMerkleTreeConfig',
                    type: {
                        defined: 'StateMerkleTreeConfig',
                    },
                },
                {
                    name: 'nullifierQueueConfig',
                    type: {
                        defined: 'NullifierQueueConfig',
                    },
                },
                {
                    name: 'additionalBytes',
                    type: 'u64',
                },
            ],
        },
        {
            name: 'appendLeavesToMerkleTrees',
            accounts: [
                {
                    name: 'feePayer',
                    isMut: true,
                    isSigner: true,
                    docs: ['Fee payer pays rollover fee.'],
                },
                {
                    name: 'authority',
                    isMut: false,
                    isSigner: true,
                    docs: [
                        'Checked whether instruction is accessed by a registered program or owner = authority.',
                    ],
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                    isOptional: true,
                    docs: [
                        'Some assumes that the Merkle trees are accessed by a registered program.',
                        'None assumes that the Merkle trees are accessed by its owner.',
                    ],
                },
                {
                    name: 'systemProgram',
                    isMut: false,
                    isSigner: false,
                },
            ],
            args: [
                {
                    name: 'leaves',
                    type: {
                        vec: {
                            defined: '(u8,[u8;32])',
                        },
                    },
                },
            ],
        },
        {
            name: 'nullifyLeaves',
            accounts: [
                {
                    name: 'authority',
                    isMut: false,
                    isSigner: true,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                    isOptional: true,
                },
                {
                    name: 'logWrapper',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'merkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'nullifierQueue',
                    isMut: true,
                    isSigner: false,
                },
            ],
            args: [
                {
                    name: 'changeLogIndices',
                    type: {
                        vec: 'u64',
                    },
                },
                {
                    name: 'leavesQueueIndices',
                    type: {
                        vec: 'u16',
                    },
                },
                {
                    name: 'leafIndices',
                    type: {
                        vec: 'u64',
                    },
                },
                {
                    name: 'proofs',
                    type: {
                        vec: {
                            vec: {
                                array: ['u8', 32],
                            },
                        },
                    },
                },
            ],
        },
        {
            name: 'insertIntoNullifierQueues',
            accounts: [
                {
                    name: 'feePayer',
                    isMut: true,
                    isSigner: true,
                    docs: ['Fee payer pays rollover fee.'],
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
                    name: 'nullifiers',
                    type: {
                        vec: {
                            array: ['u8', 32],
                        },
                    },
                },
                {
                    name: 'leafIndices',
                    type: {
                        vec: 'u32',
                    },
                },
                {
                    name: 'txHash',
                    type: {
                        option: {
                            array: ['u8', 32],
                        },
                    },
                },
                {
                    name: 'checkProofByIndex',
                    type: {
                        option: {
                            vec: 'bool',
                        },
                    },
                },
            ],
        },
        {
            name: 'rolloverStateMerkleTreeAndNullifierQueue',
            accounts: [
                {
                    name: 'feePayer',
                    isMut: true,
                    isSigner: true,
                    docs: [
                        'Signer used to receive rollover accounts rentexemption reimbursement.',
                    ],
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
                    isOptional: true,
                },
                {
                    name: 'newStateMerkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'newNullifierQueue',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'oldStateMerkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'oldNullifierQueue',
                    isMut: true,
                    isSigner: false,
                },
            ],
            args: [],
        },
        {
            name: 'batchNullify',
            accounts: [
                {
                    name: 'authority',
                    isMut: false,
                    isSigner: true,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                    isOptional: true,
                },
                {
                    name: 'logWrapper',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'merkleTree',
                    isMut: true,
                    isSigner: false,
                },
            ],
            args: [
                {
                    name: 'data',
                    type: 'bytes',
                },
            ],
        },
        {
            name: 'batchAppend',
            accounts: [
                {
                    name: 'authority',
                    isMut: false,
                    isSigner: true,
                },
                {
                    name: 'registeredProgramPda',
                    isMut: false,
                    isSigner: false,
                    isOptional: true,
                },
                {
                    name: 'logWrapper',
                    isMut: false,
                    isSigner: false,
                },
                {
                    name: 'merkleTree',
                    isMut: true,
                    isSigner: false,
                },
                {
                    name: 'outputQueue',
                    isMut: true,
                    isSigner: false,
                },
            ],
            args: [
                {
                    name: 'data',
                    type: 'bytes',
                },
            ],
        },
    ],
    accounts: [
        {
            name: 'registeredProgram',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'registeredProgramId',
                        type: 'publicKey',
                    },
                    {
                        name: 'groupAuthorityPda',
                        type: 'publicKey',
                    },
                ],
            },
        },
        {
            name: 'accessMetadata',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'owner',
                        docs: ['Owner of the Merkle tree.'],
                        type: 'publicKey',
                    },
                    {
                        name: 'programOwner',
                        docs: [
                            'Program owner of the Merkle tree. This will be used for program owned Merkle trees.',
                        ],
                        type: 'publicKey',
                    },
                    {
                        name: 'forester',
                        docs: [
                            'Optional privileged forester pubkey, can be set for custom Merkle trees',
                            'without a network fee. Merkle trees without network fees are not',
                            'forested by light foresters. The variable is not used in the account',
                            'compression program but the registry program. The registry program',
                            'implements access control to prevent contention during forester. The',
                            'forester pubkey specified in this struct can bypass contention checks.',
                        ],
                        type: 'publicKey',
                    },
                ],
            },
        },
        {
            name: 'addressMerkleTreeAccount',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'metadata',
                        type: {
                            defined: 'MerkleTreeMetadata',
                        },
                    },
                ],
            },
        },
        {
            name: 'batchedMerkleTreeMetadata',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'accessMetadata',
                        type: {
                            defined: 'AccessMetadata',
                        },
                    },
                    {
                        name: 'rolloverMetadata',
                        type: {
                            defined: 'RolloverMetadata',
                        },
                    },
                    {
                        name: 'associatedOutputQueue',
                        type: 'publicKey',
                    },
                    {
                        name: 'nextMerkleTree',
                        type: 'publicKey',
                    },
                    {
                        name: 'treeType',
                        type: 'u64',
                    },
                ],
            },
        },
        {
            name: 'batchedMerkleTreeAccount',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'metadata',
                        type: {
                            defined: 'MerkleTreeMetadata',
                        },
                    },
                    {
                        name: 'sequenceNumber',
                        type: 'u64',
                    },
                    {
                        name: 'treeType',
                        type: 'u64',
                    },
                    {
                        name: 'nextIndex',
                        type: 'u64',
                    },
                    {
                        name: 'height',
                        type: 'u32',
                    },
                    {
                        name: 'rootHistoryCapacity',
                        type: 'u32',
                    },
                    {
                        name: 'queue',
                        type: {
                            defined: 'BatchedQueue',
                        },
                    },
                ],
            },
        },
        {
            name: 'batchedQueueAccount',
            docs: [
                'Memory layout:',
                '1. QueueMetadata',
                '2. num_batches: u64',
                '3. hash_chain hash bounded vec',
                '3. for num_batches every 33 bytes is a bloom filter',
                '3. (output queue) rest of account is bounded vec',
                '',
                'One Batch account contains multiple batches.',
            ],
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'metadata',
                        type: {
                            defined: 'QueueMetadata',
                        },
                    },
                    {
                        name: 'queue',
                        type: {
                            defined: 'BatchedQueue',
                        },
                    },
                    {
                        name: 'nextIndex',
                        docs: [
                            'Output queue requires next index to derive compressed account hashes.',
                            'next_index in queue is ahead or equal to next index in the associated',
                            'batched Merkle tree account.',
                        ],
                        type: 'u64',
                    },
                ],
            },
        },
        {
            name: 'batchedQueue',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'numBatches',
                        type: 'u64',
                    },
                    {
                        name: 'batchSize',
                        type: 'u64',
                    },
                    {
                        name: 'zkpBatchSize',
                        type: 'u64',
                    },
                    {
                        name: 'currentlyProcessingBatchIndex',
                        type: 'u64',
                    },
                    {
                        name: 'nextFullBatchIndex',
                        type: 'u64',
                    },
                    {
                        name: 'bloomFilterCapacity',
                        type: 'u64',
                    },
                ],
            },
        },
        {
            name: 'groupAuthority',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'authority',
                        type: 'publicKey',
                    },
                    {
                        name: 'seed',
                        type: 'publicKey',
                    },
                ],
            },
        },
        {
            name: 'merkleTreeMetadata',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'accessMetadata',
                        type: {
                            defined: 'AccessMetadata',
                        },
                    },
                    {
                        name: 'rolloverMetadata',
                        type: {
                            defined: 'RolloverMetadata',
                        },
                    },
                    {
                        name: 'associatedQueue',
                        type: 'publicKey',
                    },
                    {
                        name: 'nextMerkleTree',
                        type: 'publicKey',
                    },
                ],
            },
        },
        {
            name: 'stateMerkleTreeAccount',
            docs: [
                'Concurrent state Merkle tree used for public compressed transactions.',
            ],
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'metadata',
                        type: {
                            defined: 'MerkleTreeMetadata',
                        },
                    },
                ],
            },
        },
        {
            name: 'queueMetadata',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'accessMetadata',
                        type: {
                            defined: 'AccessMetadata',
                        },
                    },
                    {
                        name: 'rolloverMetadata',
                        type: {
                            defined: 'RolloverMetadata',
                        },
                    },
                    {
                        name: 'associatedMerkleTree',
                        type: 'publicKey',
                    },
                    {
                        name: 'nextQueue',
                        type: 'publicKey',
                    },
                    {
                        name: 'queueType',
                        type: 'u64',
                    },
                ],
            },
        },
        {
            name: 'queueAccount',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'metadata',
                        type: {
                            defined: 'QueueMetadata',
                        },
                    },
                ],
            },
        },
        {
            name: 'rolloverMetadata',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'index',
                        docs: ['Unique index.'],
                        type: 'u64',
                    },
                    {
                        name: 'rolloverFee',
                        docs: [
                            'This fee is used for rent for the next account.',
                            'It accumulates in the account so that once the corresponding Merkle tree account is full it can be rolled over',
                        ],
                        type: 'u64',
                    },
                    {
                        name: 'rolloverThreshold',
                        docs: [
                            'The threshold in percentage points when the account should be rolled over (95 corresponds to 95% filled).',
                        ],
                        type: 'u64',
                    },
                    {
                        name: 'networkFee',
                        docs: ['Tip for maintaining the account.'],
                        type: 'u64',
                    },
                    {
                        name: 'rolledoverSlot',
                        docs: [
                            'The slot when the account was rolled over, a rolled over account should not be written to.',
                        ],
                        type: 'u64',
                    },
                    {
                        name: 'closeThreshold',
                        docs: [
                            'If current slot is greater than rolledover_slot + close_threshold and',
                            "the account is empty it can be closed. No 'close' functionality has been",
                            'implemented yet.',
                        ],
                        type: 'u64',
                    },
                    {
                        name: 'additionalBytes',
                        docs: [
                            'Placeholder for bytes of additional accounts which are tied to the',
                            'Merkle trees operation and need to be rolled over as well.',
                        ],
                        type: 'u64',
                    },
                ],
            },
        },
    ],
    types: [
        {
            name: 'ZeroOutLeafIndex',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'treeIndex',
                        type: 'u8',
                    },
                    {
                        name: 'batchIndex',
                        type: 'u8',
                    },
                    {
                        name: 'leafIndex',
                        type: 'u16',
                    },
                ],
            },
        },
        {
            name: 'AddressMerkleTreeConfig',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'height',
                        type: 'u32',
                    },
                    {
                        name: 'changelogSize',
                        type: 'u64',
                    },
                    {
                        name: 'rootsSize',
                        type: 'u64',
                    },
                    {
                        name: 'canopyDepth',
                        type: 'u64',
                    },
                    {
                        name: 'addressChangelogSize',
                        type: 'u64',
                    },
                    {
                        name: 'networkFee',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'rolloverThreshold',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'closeThreshold',
                        type: {
                            option: 'u64',
                        },
                    },
                ],
            },
        },
        {
            name: 'StateMerkleTreeConfig',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'height',
                        type: 'u32',
                    },
                    {
                        name: 'changelogSize',
                        type: 'u64',
                    },
                    {
                        name: 'rootsSize',
                        type: 'u64',
                    },
                    {
                        name: 'canopyDepth',
                        type: 'u64',
                    },
                    {
                        name: 'networkFee',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'rolloverThreshold',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'closeThreshold',
                        type: {
                            option: 'u64',
                        },
                    },
                ],
            },
        },
        {
            name: 'NullifierQueueConfig',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'capacity',
                        type: 'u16',
                    },
                    {
                        name: 'sequenceThreshold',
                        type: 'u64',
                    },
                    {
                        name: 'networkFee',
                        type: {
                            option: 'u64',
                        },
                    },
                ],
            },
        },
        {
            name: 'InitStateTreeAccountsInstructionData',
            type: {
                kind: 'struct',
                fields: [
                    {
                        name: 'index',
                        type: 'u64',
                    },
                    {
                        name: 'programOwner',
                        type: {
                            option: 'publicKey',
                        },
                    },
                    {
                        name: 'forester',
                        type: {
                            option: 'publicKey',
                        },
                    },
                    {
                        name: 'additionalBytes',
                        type: 'u64',
                    },
                    {
                        name: 'inputQueueBatchSize',
                        type: 'u64',
                    },
                    {
                        name: 'outputQueueBatchSize',
                        type: 'u64',
                    },
                    {
                        name: 'inputQueueZkpBatchSize',
                        type: 'u64',
                    },
                    {
                        name: 'outputQueueZkpBatchSize',
                        type: 'u64',
                    },
                    {
                        name: 'bloomFilterNumIters',
                        type: 'u64',
                    },
                    {
                        name: 'bloomFilterCapacity',
                        type: 'u64',
                    },
                    {
                        name: 'rootHistoryCapacity',
                        type: 'u32',
                    },
                    {
                        name: 'networkFee',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'rolloverThreshold',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'closeThreshold',
                        type: {
                            option: 'u64',
                        },
                    },
                    {
                        name: 'inputQueueNumBatches',
                        type: 'u64',
                    },
                    {
                        name: 'outputQueueNumBatches',
                        type: 'u64',
                    },
                    {
                        name: 'height',
                        type: 'u32',
                    },
                ],
            },
        },
        {
            name: 'BatchState',
            type: {
                kind: 'enum',
                variants: [
                    {
                        name: 'CanBeFilled',
                    },
                    {
                        name: 'Inserted',
                    },
                    {
                        name: 'ReadyToUpdateTree',
                    },
                ],
            },
        },
        {
            name: 'TreeType',
            type: {
                kind: 'enum',
                variants: [
                    {
                        name: 'State',
                    },
                    {
                        name: 'Address',
                    },
                    {
                        name: 'BatchedState',
                    },
                    {
                        name: 'BatchedAddress',
                    },
                ],
            },
        },
        {
            name: 'QueueType',
            type: {
                kind: 'enum',
                variants: [
                    {
                        name: 'NullifierQueue',
                    },
                    {
                        name: 'AddressQueue',
                    },
                    {
                        name: 'Input',
                    },
                    {
                        name: 'Address',
                    },
                    {
                        name: 'Output',
                    },
                ],
            },
        },
        {
            name: 'AddressQueueConfig',
            type: {
                kind: 'alias',
                value: {
                    defined: 'NullifierQueueConfig',
                },
            },
        },
    ],
    errors: [
        {
            code: 6000,
            name: 'IntegerOverflow',
            msg: 'Integer overflow',
        },
        {
            code: 6001,
            name: 'InvalidAuthority',
            msg: 'InvalidAuthority',
        },
        {
            code: 6002,
            name: 'NumberOfLeavesMismatch',
            msg: 'Leaves <> remaining accounts mismatch. The number of remaining accounts must match the number of leaves.',
        },
        {
            code: 6003,
            name: 'InvalidNoopPubkey',
            msg: 'Provided noop program public key is invalid',
        },
        {
            code: 6004,
            name: 'NumberOfChangeLogIndicesMismatch',
            msg: 'Number of change log indices mismatch',
        },
        {
            code: 6005,
            name: 'NumberOfIndicesMismatch',
            msg: 'Number of indices mismatch',
        },
        {
            code: 6006,
            name: 'NumberOfProofsMismatch',
            msg: 'NumberOfProofsMismatch',
        },
        {
            code: 6007,
            name: 'InvalidMerkleProof',
            msg: 'InvalidMerkleProof',
        },
        {
            code: 6008,
            name: 'LeafNotFound',
            msg: 'Could not find the leaf in the queue',
        },
        {
            code: 6009,
            name: 'MerkleTreeAndQueueNotAssociated',
            msg: 'MerkleTreeAndQueueNotAssociated',
        },
        {
            code: 6010,
            name: 'MerkleTreeAlreadyRolledOver',
            msg: 'MerkleTreeAlreadyRolledOver',
        },
        {
            code: 6011,
            name: 'NotReadyForRollover',
            msg: 'NotReadyForRollover',
        },
        {
            code: 6012,
            name: 'RolloverNotConfigured',
            msg: 'RolloverNotConfigured',
        },
        {
            code: 6013,
            name: 'NotAllLeavesProcessed',
            msg: 'NotAllLeavesProcessed',
        },
        {
            code: 6014,
            name: 'InvalidQueueType',
            msg: 'InvalidQueueType',
        },
        {
            code: 6015,
            name: 'InputElementsEmpty',
            msg: 'InputElementsEmpty',
        },
        {
            code: 6016,
            name: 'NoLeavesForMerkleTree',
            msg: 'NoLeavesForMerkleTree',
        },
        {
            code: 6017,
            name: 'InvalidAccountSize',
            msg: 'InvalidAccountSize',
        },
        {
            code: 6018,
            name: 'InsufficientRolloverFee',
            msg: 'InsufficientRolloverFee',
        },
        {
            code: 6019,
            name: 'UnsupportedHeight',
            msg: 'Unsupported Merkle tree height',
        },
        {
            code: 6020,
            name: 'UnsupportedCanopyDepth',
            msg: 'Unsupported canopy depth',
        },
        {
            code: 6021,
            name: 'InvalidSequenceThreshold',
            msg: 'Invalid sequence threshold',
        },
        {
            code: 6022,
            name: 'UnsupportedCloseThreshold',
            msg: 'Unsupported close threshold',
        },
        {
            code: 6023,
            name: 'InvalidAccountBalance',
            msg: 'InvalidAccountBalance',
        },
        {
            code: 6024,
            name: 'UnsupportedAdditionalBytes',
        },
        {
            code: 6025,
            name: 'InvalidGroup',
        },
        {
            code: 6026,
            name: 'ProofLengthMismatch',
        },
        {
            code: 6027,
            name: 'InvalidCommitmentLength',
            msg: 'Invalid commitment length',
        },
        {
            code: 6028,
            name: 'BloomFilterFull',
            msg: 'BloomFilterFull',
        },
        {
            code: 6029,
            name: 'BatchInsertFailed',
            msg: 'BatchInsertFailed',
        },
        {
            code: 6030,
            name: 'BatchNotReady',
            msg: 'BatchNotReady',
        },
        {
            code: 6031,
            name: 'SizeMismatch',
        },
        {
            code: 6032,
            name: 'BatchAlreadyInserted',
        },
        {
            code: 6033,
            name: 'InvalidBloomFilterCapacity',
        },
        {
            code: 6034,
            name: 'InvalidCircuitBatchSize',
        },
        {
            code: 6035,
            name: 'InvalidDiscriminator',
        },
        {
            code: 6036,
            name: 'BatchSizeNotDivisibleByZkpBatchSize',
            msg: 'batch_size is not divisible by zkp_batch_size',
        },
        {
            code: 6037,
            name: 'InclusionProofByIndexFailed',
        },
        {
            code: 6038,
            name: 'TxHashUndefined',
        },
        {
            code: 6039,
            name: 'InputDeserializationFailed',
        },
        {
            code: 6040,
            name: 'InvalidBatch',
        },
        {
            code: 6041,
            name: 'LeafIndexNotInBatch',
        },
        {
            code: 6042,
            name: 'UnsupportedParameters',
        },
    ],
};
