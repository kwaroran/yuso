use wasm_bindgen::prelude::*;
use byteorder::{ByteOrder, BigEndian};

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
                name_pointer += 1;
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
    let mut data = file.clone();
    if data[0] != 0x89 || data[1] != 0x50 || data[2] != 0x4E || data[3] != 0x47 || data[4] != 0x0D || data[5] != 0x0A || data[6] != 0x1A || data[7] != 0x0A {
        return Err(JsError::new("Invalid .png file header"));
    }

    let mut pointer:u32 = 8;
    while pointer < data.len().try_into().unwrap() {
        let usize_pointer:usize = pointer.try_into().unwrap();
        let chunk_len:u32 = BigEndian::read_u32(&data[usize_pointer..usize_pointer+4]);

        let chunk_end:u32 = pointer + chunk_len + 12;
        let chunk_name = String::from_utf8((&data[usize_pointer+4..usize_pointer+8]).to_vec()).unwrap();
        if chunk_name == "IEND" {
            return Ok(data);
        }

        if chunk_name == "tEXt"{
            data.drain(usize_pointer..(chunk_end as usize));

        }
        
        else{
            pointer = chunk_end;
        }
    }
    return Err(JsError::new("Invalid png File"));


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
        let chunk_name = String::from_utf8((&data[usize_pointer+4..usize_pointer+8]).to_vec()).unwrap();
        if chunk_name == "IEND" {
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

            return Ok(data);
        }

        if chunk_name == "tEXt"{
            data.drain(usize_pointer..(chunk_end as usize));
        }
        else{
            pointer = chunk_end;
        }
    }
    return Err(JsError::new("Invalid png File"));


}