use crate::models::*;
use ipnet::IpNet;
use std::fmt::Debug;
use std::iter::Map;
use std::net::IpAddr;
use std::slice::Iter;
use std::vec::IntoIter;

/// Network Layer Reachability Information
#[derive(Debug, PartialEq, Clone, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nlri {
    pub afi: Afi,
    pub safi: Safi,
    pub next_hop: Option<NextHopAddress>,
    pub prefixes: Vec<NetworkPrefix>,
}

impl Nlri {
    /// Returns true if this NLRI refers to the IPv4 address space.
    pub const fn is_ipv4(&self) -> bool {
        matches!(self.afi, Afi::Ipv4)
    }

    /// Returns true if this NLRI refers to the IPv6 address space.
    pub const fn is_ipv6(&self) -> bool {
        matches!(self.afi, Afi::Ipv6)
    }

    /// Returns true if this NLRI refers to reachable prefixes
    pub const fn is_reachable(&self) -> bool {
        self.next_hop.is_some()
    }

    /// Get the address of the next hop indicated by this NLRI.
    ///
    /// Panics if used on a unreachable NLRI message (ie. there is no next hop).
    pub const fn next_hop_addr(&self) -> IpAddr {
        match self.next_hop {
            Some(next_hop) => next_hop.addr(),
            None => panic!("unreachable NLRI "),
        }
    }

    pub fn new_reachable(prefix: NetworkPrefix, next_hop: Option<IpAddr>) -> Nlri {
        let next_hop = next_hop.map(NextHopAddress::from);
        let afi = match prefix.prefix {
            IpNet::V4(_) => Afi::Ipv4,
            IpNet::V6(_) => Afi::Ipv6,
        };
        let safi = Safi::Unicast;
        Nlri {
            afi,
            safi,
            next_hop,
            prefixes: vec![prefix],
        }
    }

    pub fn new_unreachable(prefix: NetworkPrefix) -> Nlri {
        let afi = match prefix.prefix {
            IpNet::V4(_) => Afi::Ipv4,
            IpNet::V6(_) => Afi::Ipv6,
        };
        let safi = Safi::Unicast;
        Nlri {
            afi,
            safi,
            next_hop: None,
            prefixes: vec![prefix],
        }
    }
}

impl IntoIterator for Nlri {
    type Item = IpNet;
    type IntoIter = Map<IntoIter<NetworkPrefix>, fn(NetworkPrefix) -> IpNet>;

    fn into_iter(self) -> Self::IntoIter {
        self.prefixes.into_iter().map(|x| x.prefix)
    }
}

impl<'a> IntoIterator for &'a Nlri {
    type Item = &'a IpNet;
    type IntoIter = Map<Iter<'a, NetworkPrefix>, fn(&NetworkPrefix) -> &IpNet>;

    fn into_iter(self) -> Self::IntoIter {
        self.prefixes.iter().map(|x| &x.prefix)
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MpReachableNlri {
    afi: Afi,
    safi: Safi,
    next_hop: NextHopAddress,
    prefixes: Vec<NetworkPrefix>,
}

impl MpReachableNlri {
    pub fn new(
        afi: Afi,
        safi: Safi,
        next_hop: NextHopAddress,
        prefixes: Vec<NetworkPrefix>,
    ) -> MpReachableNlri {
        MpReachableNlri {
            afi,
            safi,
            next_hop,
            prefixes,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MpUnreachableNlri {
    afi: Afi,
    safi: Safi,
    prefixes: Vec<NetworkPrefix>,
}

impl MpUnreachableNlri {
    pub fn new(afi: Afi, safi: Safi, prefixes: Vec<NetworkPrefix>) -> MpUnreachableNlri {
        MpUnreachableNlri {
            afi,
            safi,
            prefixes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn nlri_is_ipv4() {
        let nlri = Nlri::new_reachable(
            NetworkPrefix::from_str("10.0.2.0/24").unwrap(),
            Some("10.0.2.1".parse().unwrap()),
        );

        assert!(nlri.is_ipv4());
    }

    #[test]
    fn nlri_is_ipv6() {
        let nlri = Nlri::new_unreachable(NetworkPrefix::from_str("2001:db8::/32").unwrap());

        assert!(nlri.is_ipv6());
    }

    #[test]
    fn nlri_is_reachable() {
        let nlri = Nlri::new_reachable(
            NetworkPrefix::from_str("10.0.2.0/24").unwrap(),
            Some("10.0.2.1".parse().unwrap()),
        );

        assert!(nlri.is_reachable());
    }

    #[test]
    #[should_panic]
    fn nlri_next_hop_addr_unreachable() {
        let nlri = Nlri::new_unreachable(NetworkPrefix::from_str("10.0.2.0/24").unwrap());

        let _ = nlri.next_hop_addr();
    }

    #[test]
    fn mp_reachable_nlri_new() {
        let next_hop_addr = IpAddr::from_str("10.0.2.1").unwrap();
        let nlri = MpReachableNlri::new(
            Afi::Ipv4,
            Safi::Unicast,
            NextHopAddress::from(next_hop_addr),
            vec![NetworkPrefix::from_str("10.0.2.0/24").unwrap()],
        );

        assert_eq!(nlri.afi, Afi::Ipv4);
        assert_eq!(nlri.safi, Safi::Unicast);
        assert_eq!(nlri.next_hop.addr(), next_hop_addr);
        assert_eq!(nlri.prefixes.len(), 1);
    }

    #[test]
    fn mp_unreachable_nlri_new() {
        let nlri = MpUnreachableNlri::new(
            Afi::Ipv4,
            Safi::Unicast,
            vec![NetworkPrefix::from_str("10.0.2.0/24").unwrap()],
        );

        assert_eq!(nlri.afi, Afi::Ipv4);
        assert_eq!(nlri.safi, Safi::Unicast);
        assert_eq!(nlri.prefixes.len(), 1);
    }
}
