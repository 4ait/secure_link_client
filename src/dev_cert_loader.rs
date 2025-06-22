use include_dir::{include_dir, Dir};
use rustls::pki_types::CertificateDer;
use rustls::RootCertStore;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use x509_parser::pem::Pem;
use x509_parser::x509::X509Version;

pub struct DevCertLoader;

static DEV_CERTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/dev_certs");

static CERT_ENV : &str = "SECURE_LINK_CLIENT_DEV_PEM_CERTIFICATE";

impl DevCertLoader {

    pub async fn load_dev_certs(root_cert_store: &mut RootCertStore) -> Result<(), anyhow::Error> {
        
        for entry in DEV_CERTS_DIR.entries() {

            let file = entry.as_file().ok_or(anyhow::Error::msg("not a cert file"))?;
            let certs = Self::load_dev_cert(file.contents()).await?;
            
            for cert in certs {
                if let Err(e) = root_cert_store.add(cert) {
                    log::warn!("Failed to add dev certificate: {}", e);
                }
            }
            
        }
        
        if let Ok(dev_cert_env_value) = std::env::var(CERT_ENV) {
            Self::load_dev_cert(dev_cert_env_value.as_bytes()).await?;
        }
        
        Ok(())

    }
    
    pub async fn load_dev_cert_file(cert_path: &str, root_cert_store: &mut RootCertStore) -> Result<(), anyhow::Error> {
        
        let mut cert_file = File::open(cert_path).await?;
        let mut data = Vec::new();
        cert_file.read_to_end(&mut data).await?;
        
        let certs = Self::load_dev_cert(data.as_slice()).await?;

        for cert in certs {
            if let Err(e) = root_cert_store.add(cert) {
                log::warn!("Failed to add dev certificate: {}", e);
            }
        }
        
        Ok(())
        
    }
    
    async fn load_dev_cert(data: &[u8]) -> Result<Vec<CertificateDer>, anyhow::Error> {
        

        let der_certs: Vec<CertificateDer> =
            Pem::iter_from_buffer(&data).map(|pem| {

                let pem = pem.expect("Reading next PEM block failed");
                let x509 = pem.parse_x509().expect("X.509: decoding DER failed");

                assert_eq!(x509.tbs_certificate.version, X509Version::V3);

                CertificateDer::from(pem.contents)

            }).collect();


        Ok(der_certs)
    }
    
}