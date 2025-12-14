pub const ABC: [(char, &'static str); 26] = [
    ('A', ".-"),
    ('B', "-..."),
    ('C', "-.-."),
    ('D', "-.."),
    ('E', "."),
    ('F', "..-."),
    ('G', "--."),
    ('H', "...."),
    ('I', ".."),
    ('J', ".---"),
    ('K', "-.-"),
    ('L', ".-.."),
    ('M', "--"),
    ('N', "-."),
    ('O', "---"),
    ('P', ".--."),
    ('Q', "--.-"),
    ('R', ".-."),
    ('S', "..."),
    ('T', "-"),
    ('U', "..-"),
    ('V', "...-"),
    ('W', ".--"),
    ('X', "-..-"),
    ('Y', "-.--"),
    ('Z', "--.."),
];

pub const NUMBERS: [(char, &'static str); 10] = [
    ('1', ".----"),
    ('2', "..---"),
    ('3', "...--"),
    ('4', "....-"),
    ('5', "....."),
    ('6', "_...."),
    ('7', "--..."),
    ('8', "---.."),
    ('9', "----."),
    ('0', "-----"),
];

pub const SIGNS: [(char, &'static str); 11] = [
    ('.', ".-.-.-"),
    ('!', "-.-.--"),
    ('\'', ".----."),
    (',', "--..--"),
    ('?', "..--.."),
    ('/', "-..-."),
    ('-', "-....-"),
    ('(', "-.--.-"),
    (')', "-.--."),
    ('@', ".--.-."),
    ('&', ".-..."),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ProSign {
    AA,  // New line
    AR,  // End of message
    CT,  // Start copying
    DO,  // Change to WABUN code
    KA,  // Invitation to transmit
    KN,  // Invitation to transmit to a specific station
    SK,  // End of contact
    SN,  // Understood
    SOS, // Distress signal
    ERR, // Errorneous Transmission
}

impl ToString for ProSign {
    fn to_string(&self) -> String {
        match self {
            ProSign::AA => String::from("<AA>\n"),
            ProSign::AR => String::from("<AR> (End of Message)"),
            ProSign::CT => String::from("<CT> (Start Copying)"),
            ProSign::DO => String::from("<DO> (Change to WABUN Code)"),
            ProSign::KA => String::from("<KA> (Invitation to Transmit)"),
            ProSign::KN => String::from("<KN> (Invitation to Transmit to Specific Station)"),
            ProSign::SK => String::from("<SK> (End of Contact)"),
            ProSign::SN => String::from("<SN> (Understood)"),
            ProSign::SOS => String::from("SOS (Distress Signal)"),
            ProSign::ERR => String::from("<ERR> (Erroneous Transmission)"),
        }
    }
}

pub const PROSIGNS: [(ProSign, &'static str); 10] = [
    (ProSign::AA, ".-.-"),
    (ProSign::AR, ".-.-."),
    (ProSign::CT, "-.-.-"),
    (ProSign::DO, "-..---"),
    (ProSign::KA, "-.-.-."),
    (ProSign::KN, "-.--."),
    (ProSign::SK, "...-.-"),
    (ProSign::SN, "...-."),
    (ProSign::SOS, "...---..."),
    (ProSign::ERR, "........"),
];
