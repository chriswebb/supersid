pub fn plot_spectrum<T: super::Measurement>(spectrum: &super::SpectralDensity<T>, file_name: String, title_option: Option<&str>, x_label_option: Option<&str>, y_label_option: Option<&str>) {
    if title_option.is_none() {
        let _: complot::LinLog = (
            spectrum.data.iter()
                .map(|&super::SpectralDensitySample::<T, T>(freq, sd)| (freq.to_f64().unwrap(), vec![sd.to_f64().unwrap()])),
            complot::complot!(
                file_name,
                xlabel = match x_label_option { Some(x_label) => x_label, None => "Frequency [Hz]"},
                ylabel = match y_label_option { Some(y_label) => y_label, None => "Spectral density [s^2/Hz]" }
            ),
        ).into();
    }
    else {
        let _: complot::LogLin = (
            spectrum.data.iter()
                .map(|&super::SpectralDensitySample::<T, T>(freq, sd)| (freq.to_f64().unwrap(), vec![sd.to_f64().unwrap()])),
            complot::complot!(
                file_name,
                xlabel = match x_label_option { Some(x_label) => x_label, None => "Frequency [Hz]"},
                ylabel = match y_label_option { Some(y_label) => y_label, None => "Spectral density [s^2/Hz]" },
                title = title_option.unwrap()
            ),
        ).into();
    }
}