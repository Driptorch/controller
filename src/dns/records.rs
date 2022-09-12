use std::net::{Ipv4Addr, Ipv6Addr};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum RecordTypes {
    SOA {
        /// Time-to-live
        ttl: i32,
        /// Primary master name server
        mname: String,
        /// Email address of the administrator
        rname: String,
        /// Serial number
        serial: u32,
        /// Seconds until secondary name server refresh
        refresh: i32,
        /// Seconds after initial failure to retry from secondary name servers
        retry: i32,
        /// Seconds until secondary name servers give up if repeated failures
        expire: i32,
        /// Minimum time-to-live for negative caching
        minimum: u32
    },
    A {
        hostname: String,
        ttl: i32,
        address: Ipv4Addr
    },
    AAAA {
        hostname: String,
        ttl: i32,
        address: Ipv6Addr
    },
    CNAME {
        hostname: String,
        ttl: i32,
        cname: String
    },
    DNAME {
        hostname: String,
        ttl: i32,
        dname: String
    },
    MX {
        hostname: String,
        ttl: i32,
        preference: i16,
        exchange: String
    },
    NS {
        hostname: String,
        ttl: i32,
        nsdame: String
    },
    PTR {
        hostname: String,
        ttl: i32,
        nsdame: String
    },
    TXT {
        hostname: String,
        ttl: i32,
        txt_data: String
    },
    CAA {
        hostname: String,
        ttl: i32,
        property: String
    },
    SRV {
        hostname: String,
        ttl: i32,
        priority: u16,
        weight: u16,
        port: u16,
        target: String
    }
}