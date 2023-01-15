use super::read_ext::ReadExt;
use std::io::Read;

use super::{BucketError, LEVIN_SIGNATURE, PROTOCOL_VERSION};

use bitflags::bitflags;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum P2pCommand {
    Handshake,
    TimedSync,
    Ping,
    RequestSupportFlags,

    NotifyNewBlock,
    NotifyNewTransactions,
    NotifyRequestGetObject,
    NotifyResponseGetObject,
    NotifyRequestChain,
    NotifyResponseChainEntry,
    NotifyNewFluffyBlock,
    NotifyRequestFluffyMissingTx,
    NotifyGetTxPoolComplement,
}

impl TryFrom<u32> for P2pCommand {
    type Error = BucketError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1001 => Ok(P2pCommand::Handshake),
            1002 => Ok(P2pCommand::TimedSync),
            1003 => Ok(P2pCommand::Ping),
            1007 => Ok(P2pCommand::RequestSupportFlags),

            2001 => Ok(P2pCommand::NotifyNewBlock),
            2002 => Ok(P2pCommand::NotifyNewTransactions),
            2003 => Ok(P2pCommand::NotifyRequestGetObject),
            2004 => Ok(P2pCommand::NotifyResponseGetObject),
            2006 => Ok(P2pCommand::NotifyRequestChain),
            2007 => Ok(P2pCommand::NotifyResponseChainEntry),
            2008 => Ok(P2pCommand::NotifyNewFluffyBlock),
            2009 => Ok(P2pCommand::NotifyRequestFluffyMissingTx),
            2010 => Ok(P2pCommand::NotifyGetTxPoolComplement),

            _ => Err(BucketError::UnsupportedP2pCommand(value)),
        }
    }
}

impl From<P2pCommand> for u32 {
    fn from(val: P2pCommand) -> Self {
        match val {
            P2pCommand::Handshake => 1001,
            P2pCommand::TimedSync => 1002,
            P2pCommand::Ping => 1003,
            P2pCommand::RequestSupportFlags => 1007,

            P2pCommand::NotifyNewBlock => 2001,
            P2pCommand::NotifyNewTransactions => 2002,
            P2pCommand::NotifyRequestGetObject => 2003,
            P2pCommand::NotifyResponseGetObject => 2004,
            P2pCommand::NotifyRequestChain => 2006,
            P2pCommand::NotifyResponseChainEntry => 2007,
            P2pCommand::NotifyNewFluffyBlock => 2008,
            P2pCommand::NotifyRequestFluffyMissingTx => 2009,
            P2pCommand::NotifyGetTxPoolComplement => 2010,
        }
    }
}

bitflags! {
    pub struct Flags: u32 {
        const REQUEST = 1;
        const RESPONSE = 2;
        const START_FRAGMENT = 4;
        const END_FRAGMENT = 8;
        const DUMMY = Self::START_FRAGMENT.bits | Self::END_FRAGMENT.bits;
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BucketHead {
    pub signature: u64,
    pub size: u64,
    pub have_to_return_data: bool,
    pub command: P2pCommand,
    pub return_code: i32,
    pub flags: Flags,
    pub protocol_version: u32,
}

impl BucketHead {
    pub const SIZE: usize = 33;

    pub fn build(
        payload_size: u64,
        have_to_return_data: bool,
        command: P2pCommand,
        flags: Flags,
        return_code: i32,
    ) -> BucketHead {
        BucketHead {
            signature: LEVIN_SIGNATURE,
            size: payload_size,
            have_to_return_data,
            command,
            return_code,
            flags,
            protocol_version: PROTOCOL_VERSION,
        }
    }

    pub fn from_bytes<R: Read + ?Sized>(r: &mut R) -> Result<BucketHead, BucketError> {
        let header = BucketHead {
            signature: r.read_u64()?,
            size: r.read_u64()?,
            have_to_return_data: r.read_bool()?,
            command: P2pCommand::try_from(r.read_u32()?)?,
            return_code: r.read_i32()?,
            flags: Flags::from_bits(r.read_u32()?).ok_or(BucketError::UnknownFlags)?,
            protocol_version: r.read_u32()?,
        };

        if header.signature != LEVIN_SIGNATURE {
            return Err(BucketError::IncorrectSignature(header.signature));
        }

        if header.protocol_version != PROTOCOL_VERSION {
            return Err(BucketError::UnknownProtocolVersion(header.protocol_version));
        }

        if header.return_code < 0 {
            return Err(BucketError::Error(header.return_code));
        }

        Ok(header)
    }
}
