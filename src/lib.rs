extern crate csv;

use csv::{StringRecord, Writer};
use std::error::Error;
use std::path::Path;
use std::collections::HashMap;
use crate::errors::InvalidConstituencyName;
use regex::Regex;
use std::collections::btree_map::BTreeMap;


mod errors;

const CONSTITUENCY_REG: &str = "([০১২৩৪৫৬৭৮৯]){3}.[^:]*: (সংসদ সদস্য)";


#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct CenterDetails {
    constituency: String,
    center: String,
}

impl CenterDetails {
    fn new(constituency: &str, center: &str) -> CenterDetails {
        CenterDetails {
            constituency: constituency.to_string(),
            center: center.to_string(),
        }
    }
}

pub fn get_constituencies_translated() -> HashMap<String, String> {
    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(Path::new("res/cons_name_translate.csv"));
    let mut cons_list = HashMap::new();
    let mut b = false;
    let mut name = String::new();
    for result in rdr.unwrap().records() {
        // load the row
        let record = result.unwrap();
        if !b {
            name = record.get(0).unwrap().to_string();
        } else {
            cons_list.insert(name.clone(), record.get(0).unwrap().to_string());
        }
        b = !b;
    }
    cons_list
}

// get constituency name
pub fn get_constituency_name(line: &StringRecord, const_translated: &HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    // constituency names are located in 3rd column, ends at ':' sign
    // find to ':' and slice it to that position from 0
    line.iter()
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.to_string()
                .find(':')
                .ok_or_else(|| InvalidConstituencyName.into())
                .and_then(|u|
                    Ok(const_translated.get(&s[0..u].trim().to_string()).unwrap().clone())
                )
        }).collect()
}

// go through a CSV file and calculate total votes par symbols on
// polling centers and constituencies
pub fn aggregate_result_by_symbols(file_name: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    println!("Starting ...");
    // start reading the csv file
    let const_translate = get_constituencies_translated();
    let other_columns = vec!["মোট বৈধ", "মোট বাতিল", "প্রদত্ত ভোট", "শতকরা হার"].iter().map(|s| s.to_string().clone()).collect::<Vec<String>>();
    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(Path::new(file_name));
    // all the data are aggregated in to a map
    let mut aggregated_data: BTreeMap<CenterDetails, BTreeMap<String, String>> = BTreeMap::new();
    // list of all the symbols
    let mut symbol_list: Vec<String> = Vec::new();
    let mut constituency = String::new();
    // if next row is the list of symbols
    let mut symbol_list_row = false;
    // symbol ordering for current section
    let mut symbol_positions: Vec<String> = Vec::new();
    // go through all the lines
    for result in rdr.unwrap().records() {
        // load the row
        let record = result.unwrap();
        // check if it's a new constituency
        if is_constituency_row(&record) {
            // set the constituency name
            constituency = get_constituency_name(&record, &const_translate).unwrap();
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
        // store the votes
        let mut results = BTreeMap::new();
        get_result(&record, &symbol_positions, &mut results);
        get_other_columns(&record, &other_columns, symbol_positions.len() + 2, &mut results);
        results.insert("total_voters".to_string(), record.get(1).unwrap().to_string());
        // center
        let center = record.get(0).unwrap().trim();
        // add to output map
        aggregated_data.insert(CenterDetails::new(constituency.as_str(), center), results);
    }
    // we will add all the symbols and then generate CSV file,
    // which is why we are storing all symbols in the list
    symbol_list.sort();
    println!("Symbols found: {}", symbol_list.len());
    // now generate rows for CSV,
    let mut result_rows = Vec::new();
    aggregated_data.iter().for_each(|(k, v)| {
        let mut row = vec![
            k.constituency.clone(),
            k.center.clone(),
        ];
        // set values in the symbols
        symbol_list.iter().for_each(|symbol| {
            let votes: String = match v.get(symbol) {
                None => "0".to_string(),
                Some(i) => format!("{}", i),
            };
            row.push(votes);
        });
        other_columns.iter().for_each(|symbol| {
            let votes: String = match v.get(symbol) {
                None => "0".to_string(),
                Some(i) => format!("{}", i),
            };
            row.push(votes);
        });
        row.push(v["total_voters"].clone());
        result_rows.push(row);
    });
    println!("Rows found: {}", result_rows.len());
    // write the file to output.csv
    let mut writer = Writer::from_path(output_file).unwrap();
    let mut headers = vec!["Constituency".to_string(), "Center".to_string()];
    headers.extend(symbol_list);
    headers.extend(other_columns);
    headers.push("total_voters".to_string());
    writer.write_record(&headers);
    for result_row in result_rows {
        writer.write_record(&result_row);
    }
    Ok(())
}

fn get_other_columns(record: &StringRecord, other_columns: &Vec<String>, offset: usize, results: &mut BTreeMap<String, String>) {
    for i in 0..other_columns.len() {
        results.insert(
            other_columns[i].to_string(),
            // first 2 columns of the record are, center and total votes. Skip them
            record.get(i + offset).unwrap_or("0").to_string(),
        );
    }
}

fn get_result(record: &StringRecord, symbol_positions: &Vec<String>, results: &mut BTreeMap<String, String>) {
    for i in 0..symbol_positions.len() {
        results.insert(
            symbol_positions[i].to_string(),
            // first 2 columns of the record are, center and total votes. Skip them
            record.get(i + 2).unwrap_or("0").to_string()
        );
    }
}

pub fn check_for_new_symbols(record: &StringRecord, symbols: &mut Vec<String>) {
    for sym in record {
        let s = sym.trim().to_string();
        if s.is_empty() { continue; }
        if symbols.contains(&s) { continue; }
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
mod tests;
