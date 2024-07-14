
pub use model::*;
#[cfg(feature = "nlp")]
pub use nlp::*;

#[cfg(feature = "solvers")]
pub use solver::*;
pub use utils::*;

pub mod model;
pub mod utils;

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use std::ops::Mul;

    use approx::assert_relative_eq;
    use ndarray::array;
    use num::pow::Pow;
    use num_complex::{Complex64, ComplexFloat};

    //noinspection ALL
    #[test]
    fn test_2_1() {
        let GMRabc = 0.00744;
        let _GMRn = 0.00248;
        let rabc = 0.190;
        let _rn = 0.368;
        let Dab = 0.7622;
        let _Dbc = 1.3720;
        let _Dca = 2.1342;
        let _Dan = 1.7247;
        let _Dbn = 1.3025;
        let _Dcn = 1.5244;
        let zaa = Complex64::new(rabc + 0.0493, 0.0628 * ((1.0 / GMRabc).ln() + 8.02517));
        assert_relative_eq!(zaa.re, 0.2393, max_relative = 1e-4);
        assert_relative_eq!(zaa.im, 0.8118, max_relative = 1e-4);
        // let zaa = Complex64::new(0.2393, 0.8118);
        let zab = Complex64::new(0.0493, 0.0628 * ((1.0 / Dab).ln() + 8.02517));
        assert_relative_eq!(zab.im, 0.5210, max_relative = 1e-4);
        // let zab = Complex64::new(0.0493, 0.5210);
        let zac = Complex64::new(0.0493, 0.4564);
        let zan = Complex64::new(0.0493, 0.4698);
        let zbc = Complex64::new(0.0493, 0.4841);
        let zbn = Complex64::new(0.0493, 0.4874);
        let zcn = Complex64::new(0.0493, 0.4775);
        let znn = array![Complex64::new(0.4173, 0.8807)];
        let zij = array![[zaa, zab, zac], [zab, zaa, zbc], [zac, zbc, zaa]];
        let zin = array![[zan], [zbn], [zcn]];
        let znj = array![zan, zbn, zcn];
        let zabc = zij - zin.mul(array![Complex64::new(1.0, 0.0)] / znn).mul(znj);
        println!("{:?}", zabc);
        let a = Complex64::new(f64::cos(2.0 * PI / 3.0), f64::sin(2.0 * PI / 3.0));
        println!("as: {:?}", a);
        let matrix_as = array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
            [Complex64::new(1.0, 0.0), a * a, a],
            [Complex64::new(1.0, 0.0), a, a * a]
        ];
        println!("As: {:?}", matrix_as);
        let matrix_as_inv = array![
            [
                Complex64::new(1.0 / 3.0, 0.0),
                Complex64::new(1.0 / 3.0, 0.0),
                Complex64::new(1.0 / 3.0, 0.0)
            ],
            [
                Complex64::new(1.0 / 3.0, 0.0),
                a * Complex64::new(1.0 / 3.0, 0.0),
                a * a * Complex64::new(1.0 / 3.0, 0.0)
            ],
            [
                Complex64::new(1.0 / 3.0, 0.0),
                a * a * Complex64::new(1.0 / 3.0, 0.0),
                a * Complex64::new(1.0 / 3.0, 0.0)
            ]
        ];
        println!("As_inv {:?}", matrix_as_inv);
        println!("As_inv * As: {:?}", matrix_as_inv.dot(&matrix_as));
        let temp = matrix_as_inv.dot(&zabc);
        let z012 = temp.dot(&matrix_as);
        // let z012 = As_inv * zabc * As;
        assert_relative_eq!(z012.get([0, 0]).unwrap().re(), 0.5050, max_relative = 1e-4);
    }

    //noinspection ALL
    #[test]
    #[allow(non_snake_case)]
    fn test_2_2() {
        let GMRc = 0.005212;
        let GMRs = 0.000634;
        let dod = 3.2766;
        let _dc = 1.4402;
        let rc = 0.2548;
        let ds = 0.162814;
        let rs = 9.2411;
        let k = 13.0;
        let R = (dod - ds) / 200.;
        assert_relative_eq!(R, 0.01557, max_relative = 1e-4);
        let tmp: f64 = GMRs * k * R.pow(k - 1.0);
        let GMRcn = tmp.pow(1.0 / k);
        assert_relative_eq!(GMRcn, 0.01483, max_relative = 1e-3);
        let rcn = rs / k;
        let D12 = 0.1524;
        let D45 = 0.1524;
        let _D23 = 0.1524;
        let _D56 = 0.1524;
        let D13 = 0.3048;
        let D46 = 0.3048;
        let D14 = 0.01557;
        let _D25 = 0.01557;
        let _D36 = 0.01557;
        let D15 = 0.1524;
        let _D26 = 0.1524;
        let D16 = 0.3048;
        let z11 = Complex64::new(rc + 0.0493, 0.0628 * ((1.0 / GMRc).ln() + 8.02517));
        assert_relative_eq!(z11.re, 0.3041, max_relative = 1e-4);
        assert_relative_eq!(z11.im, 0.8341, max_relative = 1e-4);
        let z12 = Complex64::new(0.0493, 0.0628 * ((1.0 / D12).ln() + 8.02517));
        assert_relative_eq!(z12.im, 0.6221, max_relative = 1e-4);
        let z13 = Complex64::new(0.0493, 0.0628 * ((1.0 / D13).ln() + 8.02517));
        assert_relative_eq!(z13.im, 0.5786, max_relative = 1e-4);
        let z14 = Complex64::new(0.0493, 0.0628 * ((1.0 / D14).ln() + 8.02517));
        assert_relative_eq!(z14.im, 0.7654, max_relative = 1e-4);
        let z15 = Complex64::new(0.0493, 0.0628 * ((1.0 / D15).ln() + 8.02517));
        assert_relative_eq!(z15.im, 0.6221, max_relative = 1e-4);
        let z16 = Complex64::new(0.0493, 0.0628 * ((1.0 / D16).ln() + 8.02517));
        assert_relative_eq!(z16.im, 0.5786, max_relative = 1e-4);
        let z44 = Complex64::new(rcn + 0.0493, 0.0628 * ((1.0 / GMRcn).ln() + 8.02517));
        assert_relative_eq!(z44.re, 0.7602, max_relative = 1e-4);
        assert_relative_eq!(z44.im, 0.7684, max_relative = 1e-4);
        let z45 = Complex64::new(0.0493, 0.0628 * ((1.0 / D45).ln() + 8.02517));
        assert_relative_eq!(z45.im, 0.6221, max_relative = 1e-4);
        let z46 = Complex64::new(0.0493, 0.0628 * ((1.0 / D46).ln() + 8.02517));
        assert_relative_eq!(z46.im, 0.5786, max_relative = 1e-4);
    }
}
