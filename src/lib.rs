use wasm_bindgen::prelude::*;
use byteorder::{ByteOrder, BigEndian};
use crc32fast::Hasher;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn decode(data: Vec<u8>, target:String) -> Result<String,JsError> {
    if data[0] != 0x89 || data[1] != 0x50 || data[2] != 0x4E || data[3] != 0x47 || data[4] != 0x0D || data[5] != 0x0A || data[6] != 0x1A || data[7] != 0x0A {
        return Err(JsError::new("Invalid .png file header"));
    }

    let mut pointer:u32 = 8;

    while pointer < data.len().try_into().unwrap() {
        let usize_pointer:usize = pointer.try_into().unwrap();
        let chunk_len:u32 = BigEndian::read_u32(&data[usize_pointer..usize_pointer+4]);
        let chunk_end:u32 = pointer + chunk_len + 12;
        let chunk_name:String = String::from_utf8((&data[usize_pointer+4..usize_pointer+8]).to_vec()).unwrap();
        if chunk_name == "IEND" {
            return Err(JsError::new("No chara found"));
        }

        if chunk_name == "tEXt"{
            let mut name_pointer = usize_pointer+8;

            while data[name_pointer] != 0x00 {
                name_pointer += 1;
            }

            let name_data: &[u8] = &data[usize_pointer+8..name_pointer];

            let nm = String::from_utf8(name_data.to_vec()).unwrap();
            if nm == target{
                log(nm.as_str());
                name_pointer += 1;
                log(format!("{:x} {:x}", usize_pointer,chunk_end).as_str());
                let text_data = &data[name_pointer..(chunk_end as usize - 4)];
                let whole_text = String::from_utf8(text_data.to_vec()).unwrap();
    
                return Ok(whole_text);
            }
        }
        pointer = chunk_end;
    }

    return Err(JsError::new("Invalid png File"));

}


#[wasm_bindgen]
pub fn trim(file: Vec<u8>) -> Result<Vec<u8>,JsError> {
    return encode(file, "".to_string(), "".to_string());
}

#[wasm_bindgen]
pub fn encode(file: Vec<u8>, target:String, fdata: String) -> Result<Vec<u8>,JsError> {
    let mut data = file.clone();
    if data[0] != 0x89 || data[1] != 0x50 || data[2] != 0x4E || data[3] != 0x47 || data[4] != 0x0D || data[5] != 0x0A || data[6] != 0x1A || data[7] != 0x0A {
        return Err(JsError::new("Invalid .png file header"));
    }

    let mut pointer:u32 = 8;
    while pointer < data.len().try_into().unwrap() {

        let usize_pointer:usize = pointer.try_into().unwrap();
        let chunk_len:u32 = BigEndian::read_u32(&data[usize_pointer..usize_pointer+4]);
        let chunk_end:u32 = pointer + chunk_len + 12;
        let chunk_name: String = String::from_utf8((&data[usize_pointer+4..usize_pointer+8]).to_vec()).unwrap();
        if chunk_name == "IEND" {
            if target != "" {
                let mut v = "tEXt".as_bytes().to_vec();
                v.append(&mut target.as_bytes().to_vec());
                v.push(0x00);
                v.append(&mut fdata.as_bytes().to_vec());
    
                let mut checksum:Vec<u8> = crc32fast::hash(&v).to_be_bytes().to_vec();
    
                v.append(&mut checksum);
    
                let len:u32 = v.len().try_into().unwrap();
                let read_len:u32 = len - 8;
                let len_bytes = read_len.to_be_bytes();
                for byte in len_bytes.iter().rev() {
                    v.insert(0, *byte);
                }
    
                data.splice(usize_pointer..usize_pointer,v);   
            }

            if target == "" && fdata == "drop" {
                data.drain(usize_pointer..(chunk_end as usize));
            }

            return Ok(data);
        }

        if chunk_name == "tEXt" && target == ""{
            data.drain(usize_pointer..(chunk_end as usize));
        }
        else{
            pointer = chunk_end;
        }
    }
    return Err(JsError::new("Invalid png File"));


}


#[wasm_bindgen]
pub struct ChunkEncoder {
    expected_len: u32,
    real_len: u32,
    len_pointer: u32,
    hasher: Hasher,
}

#[wasm_bindgen]
impl ChunkEncoder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ChunkEncoder {
        ChunkEncoder {
            expected_len: 0,
            real_len: 0,
            len_pointer:0,
            hasher: Hasher::new()
        }
    }

    pub fn base(&mut self, filedata:Vec<u8>, target:String, size:u32) -> Result<Vec<u8>,JsError>{
        let mut vec = match encode(filedata, "".to_string(), "drop".to_string()) {
            Ok(v) => v,
            Err(e) => return Err(e)
        };


        let pointer = vec.len();
        self.len_pointer = pointer.try_into().unwrap();
        let mut v = "tEXt".as_bytes().to_vec();
        v.append(&mut target.as_bytes().to_vec());
        v.push(0x00);

        self.hasher.update(&v);

        self.real_len = v.len().try_into().unwrap();
        self.expected_len = size + self.real_len + 4;

        let len_bytes = (self.expected_len - 8).to_be_bytes();
        for byte in len_bytes.iter().rev() {
            v.insert(0, *byte);
        }

        vec.append(&mut v);

        return Ok(vec);

    }

    pub fn appendtext(&mut self, data:String) -> Vec<u8>{
        let byt = data.as_bytes();
        self.hasher.update(byt);
        self.real_len += byt.len() as u32;
        return byt.to_vec()
    }

    pub fn append(&mut self, data:Vec<u8>) -> Vec<u8>{
        self.hasher.update(&data);
        self.real_len += data.len() as u32;
        return data.to_vec()
    }

    pub fn end(&mut self) -> Result<Vec<u8>, JsError>{

        let hashsum = self.hasher.clone().finalize().to_be_bytes().to_vec();
        self.real_len += 4;

        if self.expected_len != self.real_len {
            return Err(JsError::new(format!("Length doesn't match, {}", self.expected_len - self.real_len).as_str()));
        }        
        return Ok(hashsum.to_vec());

    }
}