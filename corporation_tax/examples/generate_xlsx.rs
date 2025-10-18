use std::path::Path;
use umya_spreadsheet::*;

fn main() {
    // 日期时间格式
    let nf_datetime = NumberingFormat::default()
        .set_format_code(r#"yyyy-mm-dd hh:mm:ss"#)
        .to_owned();

    // 日期格式
    let nf_date = NumberingFormat::default()
        .set_format_code(NumberingFormat::FORMAT_DATE_YYYYMMDD2)
        .to_owned();

    // 时间格式
    let nf_time = NumberingFormat::default()
        .set_format_code(r#"hh:mm:ss"#)
        .to_owned();

    // 会计格式
    let nf_isk = NumberingFormat::default()
        .set_format_code(r#"_ [$isk]\ * #,##0.00_ ;_ [$isk]\ * \-#,##0.00_ ;_ [$isk]\ * "-"?_ ;"#)
        .to_owned();

    // 百分比
    let nf_percent = NumberingFormat::default()
        .set_format_code(r#"0.0%"#)
        .to_owned();

    println!("hello world");
    let mut book = new_file();
    let w = book.get_sheet_mut(&0).unwrap();
    w.set_name("原始数据");

    let c = w.get_cell_mut((1, 3));
    c.set_value_number(45947.0);
    c.get_style_mut().set_number_format(nf_date);

    let c = w.get_cell_mut((2, 3));
    c.set_value_number(0.1123);
    c.get_style_mut().set_number_format(nf_time);

    let c = w.get_cell_mut((3, 3));
    c.set_value_number(127.23);
    c.get_style_mut()
        .set_number_format(nf_isk)
        .set_background_color("FFFFC7CE");

    let c = w.get_cell_mut((4, 3));
    c.set_value_number(1.23);
    c.get_style_mut().set_number_format(nf_percent);

    let c = w.get_cell_mut((5, 3));
    c.set_value_number(45947.1123);
    c.get_style_mut().set_number_format(nf_datetime);

    w.add_merge_cells("A1:B2");

    w.get_column_dimension_mut("A").set_auto_width(true);
    w.get_column_dimension_mut("B").set_auto_width(true);
    w.get_column_dimension_mut("C").set_auto_width(true);
    w.get_column_dimension_mut("D").set_auto_width(true);
    w.get_column_dimension_mut("E").set_auto_width(true);

    let path = Path::new("target/cc1.xlsx");
    let r = writer::xlsx::write(&book, &path);
    if let Err(e) = r {
        println!("{:?}", e);
    }

    // let path = Path::new("target/ccc.xlsx");
    // let book = reader::xlsx::read(path).unwrap();
    // let w = book.get_sheet(&0).unwrap();
    // for col in 1..=6 {
    //     let cell = w.get_cell((col, 2)).unwrap();
    //     let cv = cell.get_cell_value();
    //     let st = cell.get_style();
    //     let nf = st.get_numbering_format();
    //     println!("{}: {:?}, {:?}", col, cv, nf);
    // }
}
