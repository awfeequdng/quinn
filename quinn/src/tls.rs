use std::fmt;

use rustls::internal::pemfile;

/// A single TLS certificate
#[derive(Debug, Clone)]
pub struct Certificate {
    pub(crate) inner: rustls::Certificate,
}

impl Certificate {
    /// Parse a DER-formatted certificate
    pub fn from_der(der: &[u8]) -> Result<Self, ParseError> {
        Ok(Self {
            inner: rustls::Certificate(der.to_vec()),
        })
    }
}

/// A chain of signed TLS certificates ending the one to be used by a server
#[derive(Debug, Clone)]
pub struct CertificateChain {
    pub(crate) certs: Vec<rustls::Certificate>,
}

impl CertificateChain {
    /// Parse a PEM-formatted certificate chain
    ///
    /// ```no_run
    /// let pem = std::fs::read("fullchain.pem").expect("error reading certificates");
    /// let cert_chain = quinn::PrivateKey::from_pem(&pem).expect("error parsing certificates");
    /// ```
    pub fn from_pem(pem: &[u8]) -> Result<Self, ParseError> {
        Ok(Self {
            certs: pemfile::certs(&mut &pem[..])
                .map_err(|()| ParseError("malformed certificate chain"))?,
        })
    }

    /// Construct a certificate chain from a list of certificates
    pub fn from_certs(certs: impl IntoIterator<Item = Certificate>) -> Self {
        certs.into_iter().collect()
    }
}

impl std::iter::FromIterator<Certificate> for CertificateChain {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Certificate>,
    {
        CertificateChain {
            certs: iter.into_iter().map(|x| x.inner).collect(),
        }
    }
}

/// The private key of a TLS certificate to be used by a server
#[derive(Debug, Clone)]
pub struct PrivateKey {
    pub(crate) inner: rustls::PrivateKey,
}

impl PrivateKey {
    /// Parse a PEM-formatted private key
    ///
    /// ```no_run
    /// let pem = std::fs::read("key.pem").expect("error reading key");
    /// let key = quinn::PrivateKey::from_pem(&pem).expect("error parsing key");
    /// ```
    pub fn from_pem(pem: &[u8]) -> Result<Self, ParseError> {
        let pkcs8 = pemfile::pkcs8_private_keys(&mut &pem[..])
            .map_err(|()| ParseError("malformed PKCS #8 private key"))?;
        if let Some(x) = pkcs8.into_iter().next() {
            return Ok(Self { inner: x });
        }
        let rsa = pemfile::rsa_private_keys(&mut &pem[..])
            .map_err(|()| ParseError("malformed PKCS #1 private key"))?;
        if let Some(x) = rsa.into_iter().next() {
            return Ok(Self { inner: x });
        }
        Err(ParseError("no private key found"))
    }

    /// Parse a DER-encoded (binary) private key
    pub fn from_der(der: &[u8]) -> Result<Self, ParseError> {
        Ok(Self {
            inner: rustls::PrivateKey(der.to_vec()),
        })
    }
}

/// Errors encountered while parsing a TLS certificate or private key
#[derive(Debug, Clone)]
pub struct ParseError(&'static str);

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.0)
    }
}
