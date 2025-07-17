use rustscript_transpiler::MATLABToRustConverter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() {
        let matlab_code = r#"
function mpc = case14
%CASE14    Power flow data for IEEE 14 bus test case.
%   This is a test comment

%% system MVA base
mpc.baseMVA = 100;

%% bus data
mpc.bus = [
    1    3       0       0    0    0    1     1.06        0    0    1    1.06    0.94;
    2    2    21.7    12.7    0    0    1    1.045    -4.98    0    1    1.06    0.94;
];

%% generator cost data
mpc.gencost = [
    2    0    0    3    0.0430292599    20    0;
    2    0    0    3            0.25    20    0;
];
"#;

        let converter = MATLABToRustConverter::new();
        let result = converter.convert_string(matlab_code);

        match result {
            Ok(rustscript_code) => {
                println!("转换结果：\n{}", rustscript_code);

                // 验证转换是否保持了原始顺序
                assert!(rustscript_code.contains("//CASE14"));
                assert!(rustscript_code.contains("baseMVA = 100;"));
                assert!(rustscript_code.contains("bus = ["));
                assert!(rustscript_code.contains("gencost = ["));

                // 验证矩阵格式是否正确
                assert!(rustscript_code.contains("[1, 3,    0,    0, 0, 0, 1,  1.06,     0, 0, 1, 1.06, 0.94]"));
            }
            Err(e) => {
                panic!("转换失败: {}", e);
            }
        }
    }

    #[test]
    fn test_order_preservation() {
        let matlab_code = r#"
% First comment
mpc.first = 1;

% Second comment
mpc.second = 2;

% Third comment
mpc.third = 3;
"#;

        let converter = MATLABToRustConverter::new();
        let result = converter.convert_string(matlab_code).unwrap();

        // 验证顺序是否保持
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.iter().position(|&line| line.contains("First comment")).unwrap() <
                lines.iter().position(|&line| line.contains("first = 1")).unwrap());
        assert!(lines.iter().position(|&line| line.contains("first = 1")).unwrap() <
                lines.iter().position(|&line| line.contains("Second comment")).unwrap());
    }
}
