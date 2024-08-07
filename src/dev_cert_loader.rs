use rustls::pki_types::CertificateDer;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use x509_parser::pem::Pem;
use x509_parser::x509::X509Version;

pub struct DevCertLoader;

impl DevCertLoader {

    pub async fn load_dev_cert(cert_path: &str) -> Result<Vec<CertificateDer>, anyhow::Error> {

        let mut cert_file = File::open(cert_path).await?;
        let mut data = Vec::new();
        cert_file.read_to_end(&mut data).await?;

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