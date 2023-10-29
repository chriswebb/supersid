use libc;
use std::{mem, panic};
use std::os::unix::io::RawFd;
use alsa::{pcm, PollDescriptors, Direction, ValueOr};
use std::ffi::CString;
use alsa::pcm::*;
mod spectral_density;
mod sound_card;

fn pcm_to_fd(p: &pcm::PCM) -> Result<RawFd, alsa::Error> {
    let mut fds: [libc::pollfd; 1] = unsafe { mem::zeroed() };
    let c = PollDescriptors::fill(p, &mut fds)?;
    if c != 1 {
        return Err(alsa::Error::unsupported("snd_pcm_poll_descriptors returned wrong number of fds"))
    }
    Ok(fds[0].fd)
}

fn record_from_plughw_standard() -> Result<(), alsa::Error> {
    let pcm = PCM::open(&*CString::new("plughw:CARD=Device,DEV=0").unwrap(), Direction::Capture, false).unwrap();
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(2).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();
    pcm.start().unwrap();
    let mut buf = [0i16; 1024];
    assert_eq!(pcm.io_i16().unwrap().readi(&mut buf).unwrap(), 1024/2);
    Ok(())
}


fn record_from_plughw_mmap() -> Result<(), alsa::Error> {
    
    use std::{thread, time};
    use alsa::direct::pcm::SyncPtrStatus;

    let pcm = PCM::open(&*CString::new("plughw:CARD=Device,DEV=0").unwrap(), Direction::Capture, false).unwrap();
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(1).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::MMapInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();

    let ss = unsafe { SyncPtrStatus::sync_ptr(pcm_to_fd(&pcm).unwrap(), false, None, None).unwrap() };
    assert_eq!(ss.state(), State::Prepared);

    let mut m = pcm.direct_mmap_capture::<i16>().unwrap();

    assert_eq!(m.status().state(), State::Prepared);
    assert_eq!(m.appl_ptr(), 0);
    assert_eq!(m.hw_ptr(), 0);


    println!("{:?}", m);

    let now = time::Instant::now();
    pcm.start().unwrap();
    while m.avail() < 256 { thread::sleep(time::Duration::from_millis(1)) };
    assert!(now.elapsed() >= time::Duration::from_millis(256 * 1000 / 44100));
    let (ptr1, md) = m.data_ptr();
    assert_eq!(ptr1.channels, 2);
    assert!(ptr1.frames >= 256);
    assert!(md.is_none());
    println!("Has {:?} frames at {:?} in {:?}", m.avail(), ptr1.ptr, now.elapsed());
    let samples: Vec<i16> = m.iter().collect();
    assert!(samples.len() >= ptr1.frames as usize * 2);
    println!("Collected {} samples", samples.len());
    let (ptr2, _md) = m.data_ptr();
    assert!(unsafe { ptr1.ptr.offset(256 * 2) } <= ptr2.ptr);
    Ok(())
}


fn main() {
    let standard_result = panic::catch_unwind(|| -> &'static str { 
        let ret = record_from_plughw_standard(); 
        if ret.is_err() {
            return "failed";
        }
        return "passed";
    });
    match standard_result {
        Ok(result_str) => println!("Recording using standard sample access {}.", result_str),
        Err(_error) => println!("Recording using standard sample access failed with panic."),
    };

    let direct_result = panic::catch_unwind(|| -> &'static str { 
        let ret = record_from_plughw_mmap(); 
        if ret.is_err() {
            return "failed";
        }
        return "passed";
    });
    
    match direct_result {
        Ok(result_str) => println!("Recording using direct sample access {}.", result_str),
        Err(_error) => println!("Recording using direct sample access failed with panic."),
    };
}
