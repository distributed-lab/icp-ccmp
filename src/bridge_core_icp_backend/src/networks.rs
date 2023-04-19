use ethabi::Uint;

pub enum Network {
    MainNet,
    Goerli,
    Sepolia
}

impl Network {
    pub fn as_str(&self) -> &'static str {
        match self {
            Network::MainNet => "mainnet",
            Network::Goerli => "goerli",
            Network::Sepolia => "sepolia"
        }
    }

    pub fn chain_id(&self) -> Uint {
        match self {
            Network::MainNet => Uint::from(1),
            Network::Goerli => Uint::from(5),
            Network::Sepolia => Uint::from(11155111),
        }
    }

    pub fn from(raw_value: &str) -> Network {
        match raw_value {
            "mainnet" => Network::MainNet,
            "goerli" => Network::Goerli,
            "sepolia" => Network::Sepolia,
            _ => panic!("unavaible network"),
        }
    }
}