use rustscript_transpiler::MATLABToRustConverter;
use std::env;
use std::process;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 如果没有参数，使用默认的case14文件进行测试
    if args.len() == 1 {
        println!("未提供参数，使用默认case14文件进行测试...");
        test_case14_conversion();
        return;
    }

    if args.len() != 3 {
        eprintln!("用法: {} <input.m> <output.txt>", args[0]);
        eprintln!("示例: {} case14.m case14_rustscript.txt", args[0]);
        eprintln!("或者直接运行 {} 来测试case14转换", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];

    println!("正在转换 {} 到 {}", input_file, output_file);

    let converter = MATLABToRustConverter::new();

    match converter.convert_file(input_file, output_file) {
        Ok(()) => {
            println!("✅ 转换成功完成！");
        }
        Err(e) => {
            eprintln!("❌ 转换失败: {}", e);
            process::exit(1);
        }
    }
}

fn test_case14_conversion() {
    let input_file = r"./rustscript-transpiler/cases/case14.m";
    let output_file = r"./rustscript-transpiler/cases/case14_converted.txt";

    println!("🔄 开始转换 case14.m 文件...");
    println!("输入文件: {}", input_file);
    println!("输出文件: {}", output_file);

    let converter = MATLABToRustConverter::new();

    match converter.convert_file(input_file, output_file) {
        Ok(()) => {
            println!("✅ 转换成功完成！");
        }
        Err(e) => {
            println!("❌ 转换失败: {}", e);
        }
    }
}


