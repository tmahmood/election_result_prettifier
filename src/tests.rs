use crate::{get_constituency_name, aggregate_result_by_symbols, check_for_new_symbols, is_constituency_row, is_center_information_row, CONSTITUENCY_REG, get_constituencies_translated, get_result, get_other_columns};
use std::fs::File;
use std::io::Read;
use csv::StringRecord;
use regex::Regex;
use std::collections::BTreeMap;

const FILE_NAME: &str = "res/test_data.csv";
const OUTPUT_FILE_NAME: &str = "res/output.csv";
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
    let p_result = vec!["হুক্কা", "হাত পাখা", "গোলাপ ফুল", "আম", "ধানের শীষ", "নৌকা", "লাঙ্গল", "কাস্তে", "মিনার"];
    assert_eq!(p_result, symbol_list);
}

#[test]
fn t_aggregate_result_by_symbols() {
    aggregate_result_by_symbols(FILE_NAME, OUTPUT_FILE_NAME);
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
    let c_list = get_constituencies_translated();
    let record = read_single_csv_row(CONST_LINE);
    let name = get_constituency_name(&record.unwrap(), &c_list);
    assert_eq!(name.unwrap(), "3 Thakurgaon-1");
}

#[test]
fn t_find_constituency_names_on_fail() {
    let c_list = get_constituencies_translated();
    let record = read_single_csv_row(CONST_LINE_WRONG);
    let res = get_constituency_name(&record.unwrap(), &c_list);
    assert!(res.is_err());
}

#[test]
fn t_find_constituency_names_with_spacing_issue() {
    let c_list = get_constituencies_translated();
    let record = read_single_csv_row(CONST_LINE_EXTRA_SPACE);
    let name = get_constituency_name(&record.unwrap(), &c_list);
    assert_eq!(name.unwrap(), "3 Thakurgaon-1");
}

#[test]
fn t_parse_result_row() {
    let symbol_positions: Vec<String> = vec!["নৌকা", "ধানের শীষ", "হাত পাখা", "মিনার"].iter().map(|k| k.to_string()).collect();
    let r = r#"1 আর কে স্টেট উচ্চ বিদ্যালয়",2885,1097,805,26,3,1931,114,2045,70.88%,,,,,,,,,,,,"#;
    let record = read_single_csv_row(r).unwrap();
    let mut results = BTreeMap::new();
    let mut result_expected = BTreeMap::new();
    result_expected.insert(symbol_positions[0].clone(), "1097".to_string());
    result_expected.insert(symbol_positions[1].clone(), "805".to_string());
    result_expected.insert(symbol_positions[2].clone(), "26".to_string());
    result_expected.insert(symbol_positions[3].clone(), "3".to_string());
    get_result(&record, &symbol_positions, &mut results);
    assert_eq!(result_expected, results);
}

#[test]
fn t_parse_other_columns() {
    use std::iter::FromIterator;
    let other_columns = vec!["মোট বৈধ", "মোট বাতিল", "প্রদত্ত ভোট", "শতকরা হার"].iter().map(|s| s.to_string().clone()).collect::<Vec<String>>();
    let symbol_positions: Vec<String> = vec!["নৌকা", "ধানের শীষ", "হাত পাখা", "মিনার"].iter().map(|k| k.to_string()).collect();
    let r = r#"1 আর কে স্টেট উচ্চ বিদ্যালয়",2885,1097,805,26,3,1931,114,2045,70.88%,,,,,,,,,,,,"#;
    let record = read_single_csv_row(r).unwrap();
    let mut result_expected = BTreeMap::from_iter(
        vec![
            (other_columns[0].clone(), "1931".to_string()),
            (other_columns[1].clone(), "114".to_string()),
            (other_columns[2].clone(), "2045".to_string()),
            (other_columns[3].clone(), "70.88%".to_string())

        ]);
    let mut results = BTreeMap::new();
    get_other_columns(&record, &other_columns, 6, &mut results);
    assert_eq!(result_expected, results);
}

fn read_single_csv_row(row: &str) -> Result<StringRecord, csv::Error> {
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(row.as_bytes());
    rdr.records().next().unwrap()
}

