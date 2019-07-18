extern crate csv;

use std::io::{Lines, Read};
use std::fs;
use csv::{Reader, ReaderBuilder, Position, StringRecord, Writer};
use std::fs::File;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::collections::HashMap;
use crate::errors::{InvalidConstituencyName, NotFamiliarRowType};
use regex::Regex;
use std::io::SeekFrom::Start;
use std::collections::btree_map::BTreeMap;


mod errors;

const CONSTITUENCY_REG: &str = "([০১২৩৪৫৬৭৮৯]){3}.[^:]*: (সংসদ সদস্য)";
const CENTER_REG: &str = "([০১২৩৪৫৬৭৮৯]){3}.[^:]*: (সংসদ সদস্য)";


#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct CenterDetails {
    constituency: String,
    center: String,
}

impl CenterDetails {
    fn new(constituency: &str, center: &str) -> CenterDetails {
        CenterDetails {
            constituency: constituency.to_string(),
            center: center.to_string()
        }
    }
}

// get constituency name
pub fn get_constituency_name(line: &StringRecord) -> Result<String, Box<Error>> {
    // constituency names are located in 3rd column, ends at ':' sign
    // find to ':' and slice it to that position from 0
    line.iter()
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.to_string()
                .find(':')
                .ok_or_else(|| InvalidConstituencyName.into())
                .and_then(|u| Ok(s[0..u].trim().to_string()))
        }).collect()
}

// go through a CSV file and calculate total votes par symbols on
// polling centers and constituencies
pub fn aggregate_result_by_symbols(filename: &str) -> Result<(), Box<Error>> {
    // start reading the csv file
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(Path::new(filename));
    // no constituencies are selected initially
    let mut state  = 0;
    let mut symbol_list : Vec<String>= Vec::new();
    let mut last_record : StringRecord = StringRecord::new();
    // start reading the records
    let mut constituency = String::new();
    let mut symbol_list_row = false;
    let mut symbol_positions: Vec<String> = Vec::new();
    let mut aggregated_data: BTreeMap<CenterDetails, BTreeMap<String, i32>> = BTreeMap::new();
    // go through all the lines
    for result in rdr.unwrap().records() {
        // load the row
        let record = result.unwrap();
        // keep a copy of the last record
        last_record = record.clone();
        // check if it's a new constituency
        if is_constituency_row(&record) {
            // set the constituency name
            constituency = get_constituency_name(&record).unwrap();
            // no need to process this row anymore
            continue;
        }
        // if center information provided, we'll get the symbols from next line
        if is_center_information_row(&record) {
            symbol_list_row = true;
            // no need to process this row anymore
            continue;
        }
        // if this row contains symbol list, get them and keep track of it
        if symbol_list_row {
            // find new symbols and add them to list
            check_for_new_symbols(&record, &mut symbol_list);
            // now go through all the symbols and mark their positions, which will be used when processing data
            symbol_positions = record.iter()
                .filter(|m| !m.is_empty())
                .map(|m| m.trim().to_string())
                .collect();
            symbol_list_row = false;
            // no need to process this row anymore
            continue;
        }
        // we want to skip empty rows
        if record.iter().all(|m| m.is_empty()) {
            // no need to process this row anymore
            continue;
        }
        // center
        let center = record.get(0).unwrap().trim();
        // total voter, not doing anything with it yet
        let total_voter = record.get(1).unwrap().parse::<i32>().unwrap();
        let mut results = BTreeMap::new();
        for i in 0..symbol_positions.len() {
            results.insert(
                symbol_positions[i].to_string(),
                record.get(i + 2).unwrap().parse::<i32>().unwrap()
            );
        }
        aggregated_data.insert(CenterDetails::new(constituency.as_str(), center), results);
    }
    // consolidate the data
    symbol_list.sort();
    let mut result_rows = Vec::new();
    aggregated_data.iter().for_each(|(k, v)| {
        let mut row = vec![
            k.constituency.clone(),
            k.center.clone(),
        ];
        symbol_list.iter().for_each(|symbol| {
            let votes: String = match v.get(symbol) {
                None => "0".to_string(),
                Some(i) => format!("{}", i),
            };
            row.push(votes);
        });
        result_rows.push(row);
    });
    // write the file to output.csv
    let mut writer = Writer::from_path("res/output.csv").unwrap();
    let mut headers = vec!["Constituency".to_string(), "Center".to_string()];
    headers.extend(symbol_list);
    writer.write_record(&headers);
    for result_row in result_rows {
        writer.write_record(&result_row);
    }
    writer.flush();
    Ok(())
}

pub fn check_for_new_symbols(record: &StringRecord, symbols: &mut Vec<String>) {
    for sym in record {
        let s = sym.trim().to_string();
        if s.is_empty() { continue }
        if symbols.contains(&s) { continue }
        symbols.push(s);
    }
}

pub fn is_constituency_row(row: &StringRecord) -> bool {
    let re = Regex::new(CONSTITUENCY_REG).unwrap();
    row.iter().any(|s| re.is_match(s))
}

pub fn is_center_information_row(row: &StringRecord) -> bool {
    row[0].starts_with("কেন্দ্র")
}


#[cfg(test)]
mod tests {
    use crate::{get_constituency_name, aggregate_result_by_symbols, check_for_new_symbols, is_constituency_row, is_center_information_row, CONSTITUENCY_REG};
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use std::collections::HashMap;
    use csv::StringRecord;
    use std::error::Error;
    use regex::Regex;

    const FILE_NAME: &str = "res/test_data.csv";
    const CONST_LINE: &str = ",,\"০০৩ ঠাকুরগাঁও-১ : সংসদ সদস্য\",,,,,,,,,,,,,,,,,,,";
    const CONST_LINE_WRONG: &str = ",,\"০০৩ ঠাকুরগাঁও-১ সংসদ সদস্য\",,,,,,,,,,,,,,,,,,,";
    const CONST_LINE_EXTRA_SPACE: &str = ",,,\"০০৩ ঠাকুরগাঁও-১ : সংসদ সদস্য\",,,,,,,,,,,,,,,,,,,";

    #[test]
    pub fn t_regexp_correct() {
        let re = Regex::new(CONSTITUENCY_REG).unwrap();
        assert!(re.is_match("০০১ This is : সংসদ সদস্য)"))
    }

    #[test]
    fn t_constituency_row() {
        const CON: &str = r#",,,"০০১ পঞ্চগড়-১ : সংসদ সদস্য",,,,,,,,,,,,,,,,,,"#;
        let r = read_single_csv_row(CON);
        assert!(is_constituency_row(&r.unwrap()));
    }

    #[test]
    fn t_center_row() {
        const CEN: &str = r#"কেন্দ্র","মোট ভোটার","আল রাশেদ প্রধান ","মোঃ আব্দুল্লাহ ","মোঃ সুমন রানা ","শেখ মোঃ- হাবিবুর রহমান ","ব্যারিস্টার মুহম্মদ নওশাদ জমির ","মোঃ মজাহারুল হক প্রধান ","মোঃ আবু সালেক ","মোট বৈধ","মোট বাতিল","প্রদত্ত ভোট","শতকরা হার",,,,,,,,,"#;
        let r = read_single_csv_row(CEN);
        assert!(is_center_information_row(&r.unwrap()));
    }

    #[test]
    fn t_finding_all_symbols() {
        let lines = vec![
            r#",,"হুক্কা ","হাত পাখা ","গোলাপ ফুল ","আম ","ধানের শীষ ","নৌকা ","লাঙ্গল ",,,,,,,,,,,,,"#,
            r#",,"হুক্কা ","আম ","হাত পাখা ","নৌকা ","কাস্তে ","ধানের শীষ ","লাঙ্গল ",,,,,,,,,,,,,"#,
            r#",,"নৌকা ","ধানের শীষ ","হাত পাখা ","মিনার ",,,,,,,,,,,,,,,,"#
        ];
        let mut symbol_list = Vec::new();
        for line in lines {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(line.as_bytes());
            let record = rdr.records().next().unwrap().unwrap();
            check_for_new_symbols(&record, &mut symbol_list)
        }
        let mut p_result = vec!["হুক্কা", "হাত পাখা", "গোলাপ ফুল","আম", "ধানের শীষ", "নৌকা", "লাঙ্গল", "কাস্তে", "মিনার"];
        assert_eq!(p_result, symbol_list);
    }

    #[test]
    fn t_aggregate_result_by_symbols() {
        let result = aggregate_result_by_symbols(FILE_NAME);
        let mut content = String::new();
        File::open("res/test_result.csv")
            .and_then(|mut f| f.read_to_string(&mut content))
            .expect("Failed to load result file");
        let mut original_result = String::new();
        File::open("res/output.csv")
            .unwrap()
            .read_to_string(&mut original_result);
        assert_eq!(content, original_result);
    }

    #[test]
    fn t_find_constituency_names() {
        let record = read_single_csv_row(CONST_LINE);
        let name = get_constituency_name(&record.unwrap());
        assert_eq!(name.unwrap(), "০০৩ ঠাকুরগাঁও-১");
    }

    #[test]
    fn t_find_constituency_names_on_fail() {
        let record = read_single_csv_row(CONST_LINE_WRONG);
        let res = get_constituency_name(&record.unwrap());
        assert!(res.is_err());
    }

    #[test]
    fn t_find_constituency_names_with_spacing_issue() {
        let record = read_single_csv_row(CONST_LINE_EXTRA_SPACE);
        let name = get_constituency_name(&record.unwrap());
        assert_eq!(name.unwrap(), "০০৩ ঠাকুরগাঁও-১");
    }

    fn read_single_csv_row(row: &str) -> Result<StringRecord, csv::Error> {
        let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(row.as_bytes());
        rdr.records().next().unwrap()
    }

}
