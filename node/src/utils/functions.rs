use crate::data_access::data_access::DataAccess;
use crate::data_access::data_access_handler::DataAccessHandler;
use crate::meta_data::clients::meta_data_client::ClientMetaDataAcces;
use crate::meta_data::keyspaces::keyspace_meta_data_acces::KeyspaceMetaDataAccess;
use crate::meta_data::meta_data_handler::MetaDataHandler;
use crate::meta_data::nodes::node_meta_data_acces::NodesMetaDataAccess;
use crate::parsers::tokens::data_type::DataType;
use crate::queries::where_logic::where_clause::WhereClause;
use crate::utils::constants::{CLIENT_METADATA_PATH, IP_FILE, KEYSPACE_METADATA_PATH};
use crate::utils::errors::Errors;
use crate::utils::errors::Errors::ServerError;
use crate::utils::types::node_ip::NodeIp;
use crate::utils::types::primary_key::PrimaryKey;
use openssl::rand::rand_bytes;
use openssl::symm::{decrypt, encrypt, Cipher};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

pub fn get_long_string_from_str(str: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice((str.len() as u32).to_be_bytes().as_ref());
    bytes.extend_from_slice(str.as_bytes());
    bytes
}

pub fn check_table_name(table_name: &String) -> Result<String, Errors> {
    use_client_meta_data(|client_meta_data| {
        if table_name.is_empty() {
            return Err(Errors::SyntaxError(String::from("Table is empty")));
        }
        if !table_name.contains('.')
            && client_meta_data
                .get_keyspace(CLIENT_METADATA_PATH.to_string())?
                .is_none()
        {
            return Err(Errors::SyntaxError(String::from(
                "Keyspace not defined and non keyspace in usage",
            )));
        } else if table_name.contains('.') {
            return Ok(table_name.to_string());
        };
        let Some(kp) = client_meta_data.get_keyspace(CLIENT_METADATA_PATH.to_string())? else {
            return Err(Errors::SyntaxError(String::from("Keyspace not in usage")));
        };
        Ok(format!("{}.{}", kp, table_name))
    })
}

pub fn get_columns_from_table(table_name: &str) -> Result<HashMap<String, DataType>, Errors> {
    let binding = table_name.split('.').collect::<Vec<&str>>();
    let identifiers = &binding.as_slice();
    use_keyspace_meta_data(|handler| {
        handler.get_columns_type(
            KEYSPACE_METADATA_PATH.to_string(),
            identifiers[0],
            identifiers[1],
        )
    })
}

pub fn get_table_primary_key(table_name: &str) -> Result<PrimaryKey, Errors> {
    let (keyspace, table) = split_keyspace_table(table_name)?;
    use_keyspace_meta_data(|handler| {
        handler.get_primary_key(KEYSPACE_METADATA_PATH.to_string(), keyspace, table)
    })
}

pub fn get_table_pk(table_name: &str) -> Result<HashSet<String>, Errors> {
    Ok(get_table_primary_key(table_name)?.get_full_pk_in_hash())
}

pub fn get_table_partition(table_name: &str) -> Result<HashSet<String>, Errors> {
    Ok(get_table_primary_key(table_name)?
        .partition_keys
        .into_iter()
        .collect::<HashSet<String>>())
}
pub fn get_table_clustering_columns(table_name: &str) -> Result<HashSet<String>, Errors> {
    Ok(get_table_primary_key(table_name)?
        .clustering_columns
        .into_iter()
        .collect::<HashSet<String>>())
}

pub fn split_keyspace_table(input: &str) -> Result<(&str, &str), Errors> {
    let mut parts = input.split('.');
    let keyspace = parts
        .next()
        .ok_or_else(|| Errors::SyntaxError("Missing keyspace".to_string()))?;
    let table = parts
        .next()
        .ok_or_else(|| Errors::SyntaxError("Missing table".to_string()))?;
    if parts.next().is_some() {
        return Err(Errors::SyntaxError(
            "Too many parts, expected only keyspace.table".to_string(),
        ));
    }
    Ok((keyspace, table))
}

pub fn get_int_from_string(string: &String) -> Result<i32, Errors> {
    string
        .parse::<i32>()
        .map_err(|_| Errors::SyntaxError(format!("Could not parse int: {}", string)))
}

pub fn get_partition_key_from_where(
    table_name: &str,
    where_clause: &Option<WhereClause>,
) -> Result<Vec<String>, Errors> {
    let Some(where_clause) = where_clause else {
        return Err(Errors::SyntaxError(String::from(
            "Where clause must be defined",
        )));
    };
    let mut partition_key = Vec::new();
    let table_partition = get_table_partition(table_name)?;
    where_clause.get_primary_key(&mut partition_key, &table_partition)?;
    if partition_key.len() != table_partition.len() {
        return Err(Errors::SyntaxError(String::from(
            "Full partition key must be defined in where clause",
        )));
    }
    Ok(partition_key)
}

pub fn get_own_ip() -> Result<NodeIp, Errors> {
    let content = fs::read_to_string(IP_FILE).map_err(|e| ServerError(e.to_string()))?;
    let split = content.split(":").collect::<Vec<&str>>();
    let port = split[1].parse::<u16>().unwrap();
    NodeIp::new_from_string(split[0], port)
}

pub fn start_listener<F>(socket: SocketAddr, handle_connection: F) -> Result<(), Errors>
where
    F: Fn(&mut TcpStream) -> Result<(), Errors>,
{
    let listener = bind_listener(socket)?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => handle_connection(&mut stream)?,
            Err(_) => return Err(ServerError(String::from("Failed to connect to listener"))),
        }
    }
    Ok(())
}

pub fn flush_stream(stream: &mut TcpStream) -> Result<(), Errors> {
    stream
        .flush()
        .map_err(|_| ServerError(String::from("Failed to flush stream")))
}

const AES_KEY: [u8; 32] = [
    107, 133, 195, 73, 171, 146, 174, 177, 245, 55, 2, 116, 4, 202, 100, 1, 75, 15, 151, 34, 194,
    240, 98, 3, 111, 115, 214, 153, 82, 205, 149, 103,
];

pub fn write_to_stream(stream: &mut TcpStream, content: &[u8]) -> Result<(), Errors> {
    let (encrypted_data, iv) = encrypt_message(content, &AES_KEY)?;
    let mut message = iv.clone();
    message.extend(encrypted_data);
    stream
        .write_all(&message)
        .map_err(|_| ServerError(String::from("Failed to write to stream")))
}

pub fn read_exact_from_stream(stream: &mut TcpStream) -> Result<Vec<u8>, Errors> {
    let mut buffer = [0; 1024];
    let size = stream
        .read(&mut buffer)
        .map_err(|_| ServerError(String::from("Failed to read stream")))?;
    if size < 16 {
        return Ok(Vec::new());
    }
    let iv = &buffer[..16];
    let encrypted_data = &buffer[16..size];

    decrypt_message(encrypted_data, iv, &AES_KEY)
}

pub fn read_from_stream_no_zero(stream: &mut TcpStream) -> Result<Vec<u8>, Errors> {
    let buf = read_exact_from_stream(stream)?;
    if buf.is_empty() {
        return Err(ServerError(String::from("Empty stream")));
    }
    Ok(buf)
}

fn encrypt_message(message: &[u8], aes_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Errors> {
    let cipher = Cipher::aes_256_cbc();
    let iv = generate_iv()?;
    let encrypted = encrypt(cipher, aes_key, Some(&iv), message)
        .map_err(|_| ServerError(String::from("Failed to write to stream")))?;
    Ok((encrypted, iv))
}

fn decrypt_message(encrypted_message: &[u8], iv: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, Errors> {
    let cipher = Cipher::aes_256_cbc();
    let decrypted_data = decrypt(cipher, aes_key, Some(iv), encrypted_message)
        .map_err(|_| ServerError(String::from("Failed to read to stream")))?;
    Ok(decrypted_data)
}

fn generate_iv() -> Result<Vec<u8>, Errors> {
    let mut iv = vec![0; 16];
    rand_bytes(&mut iv).map_err(|_| ServerError(String::from("Failed to write to stream")))?;
    Ok(iv)
}

pub fn serialize_to_string<T: Serialize>(object: &T) -> Result<String, Errors> {
    serde_json::to_string(&object).map_err(|_| ServerError("Failed to serialize data".to_string()))
}

pub fn deserialize_from_slice<T: DeserializeOwned>(data: &[u8]) -> Result<T, Errors> {
    serde_json::from_slice(data).map_err(|_| ServerError("Failed to deserialize data".to_string()))
}

pub fn deserialize_from_str<T: DeserializeOwned>(data: &str) -> Result<T, Errors> {
    serde_json::from_str(data).map_err(|_| ServerError("Failed to deserialize data".to_string()))
}

pub fn bind_listener(socket_addr: SocketAddr) -> Result<TcpListener, Errors> {
    TcpListener::bind(socket_addr).map_err(|_| ServerError(String::from("Failed to set listener")))
}

pub fn connect_to_socket(socket_addr: SocketAddr) -> Result<TcpStream, Errors> {
    TcpStream::connect(socket_addr)
        .map_err(|_| ServerError(String::from("Error connecting to socket.")))
}

pub fn use_node_meta_data<F, T>(action: F) -> Result<T, Errors>
where
    F: FnOnce(&NodesMetaDataAccess) -> Result<T, Errors>,
{
    let mut meta_data_stream = MetaDataHandler::establish_connection()?;
    let node_metadata =
        MetaDataHandler::get_instance(&mut meta_data_stream)?.get_nodes_metadata_access();
    action(&node_metadata)
}

pub fn use_keyspace_meta_data<F, T>(action: F) -> Result<T, Errors>
where
    F: FnOnce(&KeyspaceMetaDataAccess) -> Result<T, Errors>,
{
    let mut meta_data_stream = MetaDataHandler::establish_connection()?;
    let keyspace_metadata =
        MetaDataHandler::get_instance(&mut meta_data_stream)?.get_keyspace_meta_data_access();
    action(&keyspace_metadata)
}

pub fn use_client_meta_data<F, T>(action: F) -> Result<T, Errors>
where
    F: FnOnce(&ClientMetaDataAcces) -> Result<T, Errors>,
{
    let mut meta_data_stream = MetaDataHandler::establish_connection()?;
    let client_metadata =
        MetaDataHandler::get_instance(&mut meta_data_stream)?.get_client_meta_data_access();
    action(&client_metadata)
}

pub fn use_data_access<F, T>(action: F) -> Result<T, Errors>
where
    F: FnOnce(&DataAccess) -> Result<T, Errors>,
{
    let mut meta_data_stream = DataAccessHandler::establish_connection()?;
    let data_access = DataAccessHandler::get_instance(&mut meta_data_stream)?;
    action(&data_access)
}

pub fn write_all_to_file(file: &mut File, content: &[u8]) -> Result<(), Errors> {
    file.write_all(content)
        .map_err(|_| ServerError(String::from("Failed to write to file")))
}
