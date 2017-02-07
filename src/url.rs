const SPACE: u8 = 32;
const PERCENT: u8 = 37;
const PLUS: u8 = 43;
const ZERO: u8 = 48;
const NINE: u8 = 57;
const UA: u8 = 65;
const UF: u8 = 70;
const UZ: u8 = 90;
const LA: u8 = 97;
const LF: u8 = 102;
const LZ: u8 = 122;

pub fn encode_percent(str: &[u8]) -> Vec<u8> {
	let mut result: Vec<u8> = Vec::new();
	for &x in str.iter() {
		match x {
			ZERO ... NINE => { result.push(x); },
			UA ... UZ => { result.push(x); },
			LA ... LZ => { result.push(x); },
			_ => {
				let msb = x >> 4 & 0x0F;
				let lsb = x & 0x0F;
				result.push(PERCENT);
				result.push(if msb < 10 { msb + ZERO } else { msb - 10 + UA });
				result.push(if lsb < 10 { lsb + ZERO } else { lsb - 10 + UA });
			},
		}
	}
	result
}

pub fn decode_percent(str: &[u8]) -> Vec<u8> {
	let mut result: Vec<u8> = Vec::new();
	str.iter().fold((0, 0), |(flag, sum), &x|
		match x {
			PLUS => { result.push(SPACE); (0, 0) },
			PERCENT => (1, 0),
			ZERO ... NINE => {
				match flag {
					1 => (2, x - ZERO),
					2 => { result.push(sum * 16 + (x - ZERO)); (0, 0) },
					_ => { result.push(x); (0, 0) },
				}
			},
			UA ... UF => {
				match flag {
					1 => (2, x - UA + 10),
					2 => { result.push(sum * 16 + (x - UA) + 10); (0, 0) },
					_ => { result.push(x); (0, 0) },
				}
			},
			LA ... LF => {
				match flag {
					1 => (2, x - LA + 10),
					2 => { result.push(sum * 16 + (x - LA) + 10); (0, 0) },
					_ => { result.push(x); (0, 0) },
				}
			},
			_ => { result.push(x); (0, 0) },
		}
	);
	result
}

#[cfg(test)]
mod tests {
	use std::str;
	use super::encode_percent;
	use super::decode_percent;
	#[test]
	fn test_encode_percent() {
		assert_eq!("%E3%81%82%E3%81%84%E3%81%86%E3%81%88%E3%81%8A", str::from_utf8(encode_percent("あいうえお".as_bytes()).as_slice()).unwrap());
		assert_eq!("%E3%81%8B%E3%81%8D%E3%81%8F%E3%81%91%E3%81%93", str::from_utf8(encode_percent("かきくけこ".as_bytes()).as_slice()).unwrap());
		assert_eq!("%E3%81%95%E3%81%97%E3%81%99%E3%81%9B%E3%81%9D", str::from_utf8(encode_percent("さしすせそ".as_bytes()).as_slice()).unwrap());
		assert_eq!("%E3%81%9F%E3%81%A1%E3%81%A4%E3%81%A6%E3%81%A8", str::from_utf8(encode_percent("たちつてと".as_bytes()).as_slice()).unwrap());
		assert_eq!("%E3%81%AA%E3%81%AB%E3%81%AC%E3%81%AD%E3%81%AE", str::from_utf8(encode_percent("なにぬねの".as_bytes()).as_slice()).unwrap());
	}
	#[test]
	fn test_decode_percent() {
		assert_eq!("あいうえお", str::from_utf8(decode_percent(b"%E3%81%82%E3%81%84%E3%81%86%E3%81%88%E3%81%8A").as_slice()).unwrap());
		assert_eq!("かきくけこ", str::from_utf8(decode_percent(b"%E3%81%8B%E3%81%8D%E3%81%8F%E3%81%91%E3%81%93").as_slice()).unwrap());
		assert_eq!("さしすせそ", str::from_utf8(decode_percent(b"%E3%81%95%E3%81%97%E3%81%99%E3%81%9B%E3%81%9D").as_slice()).unwrap());
		assert_eq!("たちつてと", str::from_utf8(decode_percent(b"%E3%81%9F%E3%81%A1%E3%81%A4%E3%81%A6%E3%81%A8").as_slice()).unwrap());
		assert_eq!("なにぬねの", str::from_utf8(decode_percent(b"%E3%81%AA%E3%81%AB%E3%81%AC%E3%81%AD%E3%81%AE").as_slice()).unwrap());
	}
}
