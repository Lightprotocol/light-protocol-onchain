use groth16_solana::groth16::Groth16Verifyingkey;

pub const VERIFYINGKEY: Groth16Verifyingkey =  Groth16Verifyingkey {
	nr_pubinputs: 5,

	vk_alpha_g1: [
		45,77,154,167,227,2,217,223,65,116,157,85,7,148,157,5,219,234,51,251,177,108,100,59,34,245,153,162,190,109,242,226,
		20,190,221,80,60,55,206,176,97,216,236,96,32,159,227,69,206,137,131,10,25,35,3,1,240,118,202,255,0,77,25,38,
	],

	vk_beta_g2: [
		9,103,3,47,203,247,118,209,175,201,133,248,136,119,241,130,211,132,128,166,83,242,222,202,169,121,76,188,59,243,6,12,
		14,24,120,71,173,76,121,131,116,208,214,115,43,245,1,132,125,214,139,192,224,113,36,30,2,19,188,127,193,61,183,171,
		48,76,251,209,224,138,112,74,153,245,232,71,217,63,140,60,170,253,222,196,107,122,13,55,157,166,154,77,17,35,70,167,
		23,57,193,177,164,87,168,199,49,49,35,210,77,47,145,146,248,150,183,198,62,234,5,169,213,127,6,84,122,208,206,200,
	],

	vk_gamme_g2: [
		25,142,147,147,146,13,72,58,114,96,191,183,49,251,93,37,241,170,73,51,53,169,231,18,151,228,133,183,174,243,18,194,
		24,0,222,239,18,31,30,118,66,106,0,102,94,92,68,121,103,67,34,212,247,94,218,221,70,222,189,92,217,146,246,237,
		9,6,137,208,88,95,240,117,236,158,153,173,105,12,51,149,188,75,49,51,112,179,142,243,85,172,218,220,209,34,151,91,
		18,200,94,165,219,140,109,235,74,171,113,128,141,203,64,143,227,209,231,105,12,67,211,123,76,230,204,1,102,250,125,170,
	],

	vk_delta_g2: [
		11,72,217,56,151,67,199,214,67,43,111,63,235,146,24,169,93,127,233,220,48,137,215,0,151,112,13,86,130,158,151,70,
		23,232,135,231,173,87,212,212,98,77,192,95,135,150,243,226,216,217,240,213,16,253,3,161,166,160,76,197,186,249,97,65,
		40,105,88,149,107,52,182,144,1,108,219,5,24,23,89,234,140,75,17,162,158,204,117,41,96,202,94,39,207,244,135,49,
		29,59,111,109,36,185,149,67,115,106,76,222,244,196,224,237,130,134,235,16,50,14,200,131,164,17,84,228,66,197,65,35,
	],

	vk_ic: &[
		[
			25,151,87,193,47,88,160,67,173,221,213,135,233,11,228,146,255,163,202,23,131,16,200,13,19,203,26,106,170,22,4,163,
			22,20,58,224,159,174,126,223,68,84,199,141,123,175,206,143,144,19,163,229,63,88,248,76,80,209,36,28,48,236,73,12,
		],
		[
			39,114,213,222,193,185,28,178,192,136,218,130,30,10,75,224,9,42,20,227,48,190,154,45,245,13,209,175,137,66,125,131,
			19,157,205,106,134,41,20,66,225,115,253,51,80,199,72,177,140,48,8,218,14,9,71,65,119,3,55,89,249,104,63,184,
		],
		[
			16,165,185,218,200,145,148,228,19,86,2,241,36,86,52,88,46,38,237,246,148,105,15,20,18,111,163,73,11,52,39,239,
			38,209,205,192,104,234,67,186,223,224,202,221,212,136,160,232,252,236,156,72,126,157,113,100,20,198,209,111,185,233,58,44,
		],
		[
			13,79,149,28,25,229,120,140,120,224,116,154,92,10,19,75,142,187,142,183,202,231,227,165,19,94,120,193,30,39,77,4,
			30,32,61,11,151,59,90,109,86,204,106,8,252,65,169,58,123,168,147,126,66,218,205,71,166,85,56,218,154,116,114,218,
		],
		[
			41,173,168,236,252,165,24,97,179,202,167,48,114,247,211,90,82,4,132,112,35,160,95,51,230,49,107,118,64,170,13,210,
			21,197,105,157,63,50,216,201,16,45,122,135,179,9,246,102,117,210,218,69,22,192,66,53,227,241,118,117,17,126,47,148,
		],
	]
};