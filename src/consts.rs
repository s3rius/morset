use std::fmt;

pub const ABC: [(char, &str); 26] = [
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

pub const NUMBERS: [(char, &str); 10] = [
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

pub const SIGNS: [(char, &str); 11] = [
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
#[allow(clippy::upper_case_acronyms)]
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

impl fmt::Display for ProSign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProSign::AA => writeln!(f, "<AA>"),
            ProSign::AR => write!(f, "<AR> (End of Message)"),
            ProSign::CT => write!(f, "<CT> (Start Copying)"),
            ProSign::DO => write!(f, "<DO> (Change to WABUN Code)"),
            ProSign::KA => write!(f, "<KA> (Invitation to Transmit)"),
            ProSign::KN => write!(f, "<KN> (Invitation to Transmit to Specific Station)"),
            ProSign::SK => write!(f, "<SK> (End of Contact)"),
            ProSign::SN => write!(f, "<SN> (Understood)"),
            ProSign::SOS => write!(f, "SOS (Distress Signal)"),
            ProSign::ERR => write!(f, "<ERR> (Erroneous Transmission)"),
        }
    }
}

pub const PROSIGNS: [(ProSign, &str); 10] = [
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
