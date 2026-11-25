use anyhow::Result;
use serde::Deserialize;

const DATACENTER_RULES_URL: &str = "https://eu.api.ovh.com/v1/vps/order/rule/datacenter";
const OUT_OF_STOCK: &str = "out-of-stock";

#[derive(Deserialize)]
struct DatacenterRules {
    datacenters: Vec<Datacenter>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Datacenter {
    pub datacenter: String,
    pub code: String,
    pub status: String,
    #[serde(default)]
    pub linux_status: String,
    #[serde(default)]
    pub windows_status: String,
}

impl Datacenter {
    pub fn is_available(&self) -> bool {
        self.status != OUT_OF_STOCK
    }

    pub fn is_european(&self) -> bool {
        self.code.starts_with("eu-")
    }

    pub fn location(&self) -> String {
        let name = match self.code.as_str() {
            "eu-west-gra" => "Gravelines (France)",
            "eu-west-sbg" => "Strasbourg (France)",
            "eu-west-rbx" => "Roubaix (France)",
            "eu-west-lim" => "Limburg (Allemagne)",
            "eu-west-eri" => "Erith / Londres (Royaume-Uni)",
            "eu-central-waw" => "Varsovie (Pologne)",
            "eu-south-mil" => "Milan (Italie)",
            _ => return format!("{} ({})", self.datacenter, self.code),
        };
        name.to_owned()
    }
}

pub struct OvhClient {
    http: reqwest::Client,
    subsidiary: String,
}

impl OvhClient {
    pub fn new(http: reqwest::Client, subsidiary: String) -> Self {
        Self { http, subsidiary }
    }

    pub async fn datacenters(&self, model: u8) -> Result<Vec<Datacenter>> {
        let plan_code = format!("vps-2025-model{model}");
        let rules = self
            .http
            .get(DATACENTER_RULES_URL)
            .query(&[
                ("ovhSubsidiary", self.subsidiary.as_str()),
                ("planCode", plan_code.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<DatacenterRules>()
            .await?;

        Ok(rules.datacenters)
    }
}
