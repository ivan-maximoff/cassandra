use super::{cluster::Cluster, node::Node};
use crate::{
    meta_data::keyspaces::keyspace_meta_data_acces::KeyspaceMetaDataAccess,
    utils::{constants::KEYSPACE_METADATA, errors::Errors},
};
use murmur3::murmur3_32;
use std::{
    fs::{File, OpenOptions},
    io::Cursor,
};
use crate::utils::node_ip::NodeIp;

#[derive(Debug)]
pub struct NodesMetaDataAccess;

impl NodesMetaDataAccess {
    fn open(path: &str) -> Result<File, Errors> {
        let file = OpenOptions::new()
            .read(true) // Permitir lectura
            .write(true) // Permitir escritura
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|_| Errors::ServerError("Unable to open or create file".to_string()))?;
        Ok(file)
    }

    fn read_cluster(path: &str) -> Result<Cluster, Errors> {
        let file = Self::open(path)?;
        let cluster: Cluster = serde_json::from_reader(&file).map_err(|_| {
            Errors::ServerError("Failed to read or deserialize Cluster".to_string())
        })?;
        Ok(cluster)
    }

    pub fn get_full_nodes_list(&self, path: &str) -> Result<Vec<Node>, Errors> {
        let cluster = Self::read_cluster(path)?;
        let mut nodes_list = Vec::new();
        for node in cluster.get_other_nodes() {
            nodes_list.push(Node::new_from_node(node))
        }
        nodes_list.push(Node::new_from_node(cluster.get_own_node()));
        Ok(nodes_list)
    }

    pub fn get_cluster(&self, path: &str) -> Result<Cluster, Errors> {
        Self::read_cluster(path)
    }

    pub fn write_cluster(path: &str, cluster: &Cluster) -> Result<(), Errors> {
        //let file = Self::open(path)?;
        let file = File::create(path)
            .map_err(|_| Errors::ServerError("Failed to open file for writing".to_string()))?;
        serde_json::to_writer(&file, &cluster)
            .map_err(|_| Errors::ServerError("Failed to write Cluster to file".to_string()))?;
        Ok(())
    }

    pub fn set_new_cluster(&self, path: &str, cluster: &Cluster) -> Result<(), Errors> {
        Self::write_cluster(path, cluster)
    }

    pub fn get_own_ip(&self, path: &str) -> Result<NodeIp, Errors> {
        let cluster = Self::read_cluster(path)?;
        Ok(NodeIp::new_from_ip(cluster.get_own_ip()))
    }
    pub fn get_own_ip_(path: &str) -> Result<NodeIp, Errors> {
        let cluster = Self::read_cluster(path)?;
        Ok(NodeIp::new_from_ip(cluster.get_own_ip()))
    }

    // pub fn get_own_port(&self, path: &str) -> Result<String, Errors> {
    //     let cluster = Self::read_cluster(path)?;
    //     Ok(cluster.get_own_port().to_string())
    // }
    // pub fn get_own_port_(path: &str) -> Result<String, Errors> {
    //     let cluster = Self::read_cluster(path)?;
    //     Ok(cluster.get_own_port().to_string())
    // }

    pub fn set_inactive(&self, path: &str, ip: &NodeIp) -> Result<(), Errors> {
        let cluster = NodesMetaDataAccess::read_cluster(path)?;
        let mut nodes_list = Vec::new();
        for node in cluster.get_other_nodes() {
            if node.get_ip() != ip {
                nodes_list.push(Node::new_from_node(node))
            } else {
                let mut inactive = Node::new_from_node(node);
                inactive.set_inactive();
                nodes_list.push(inactive);
            }
        }
        let new_cluster = Cluster::new(Node::new_from_node(cluster.get_own_node()), nodes_list);
        Self::write_cluster(path, &new_cluster)?;
        Ok(())
    }

    pub fn set_active(&self, path: &str, node_pos: usize) -> Result<(), Errors> {
        let cluster = NodesMetaDataAccess::read_cluster(path)?;
        let mut nodes_list = Vec::new();
        for node in cluster.get_other_nodes() {
            if node.get_pos() != node_pos {
                nodes_list.push(Node::new_from_node(node))
            } else {
                let mut inactive = Node::new_from_node(node);
                inactive.set_active();
                nodes_list.push(inactive);
            }
        }
        let new_cluster = Cluster::new(Node::new_from_node(cluster.get_own_node()), nodes_list);
        Self::write_cluster(path, &new_cluster)?;
        Ok(())
    }

    pub fn get_partition_full_ips(
        &self,
        path: &str,
        primary_key: &Option<Vec<String>>,
        keyspace: String,
    ) -> Result<Vec<NodeIp>, Errors> {
        let cluster = Self::read_cluster(path)?;
        if let Some(primary_key) = primary_key {
            let hashing_key = hash_string_murmur3(&primary_key.join(""));
            let pos = hashing_key % cluster.len_nodes() + 1;
            let keyspace_metadata = KeyspaceMetaDataAccess {};
            let replication =
                keyspace_metadata.get_replication(KEYSPACE_METADATA.to_owned(), &keyspace)?;
            cluster.get_nodes(pos, replication)
        } else {
            cluster.get_all_ips()
        }
    }

    pub fn append_new_node(&self, path: &str, new_node: Node) -> Result<(), Errors> {
        let mut cluster = NodesMetaDataAccess::read_cluster(path)?;
        cluster.append_new_node(new_node);
        Self::write_cluster(path, &cluster)?;
        Ok(())
    }

    pub fn get_nodes_quantity(&self, path: &str) -> Result<usize, Errors> {
        let cluster = NodesMetaDataAccess::read_cluster(path)?;
        Ok(cluster.len_nodes())
    }
}

fn hash_string_murmur3(input: &str) -> usize {
    let mut buffer = Cursor::new(input.as_bytes());
    let hash = murmur3_32(&mut buffer, 0).expect("Unable to compute hash");
    hash as usize
}
