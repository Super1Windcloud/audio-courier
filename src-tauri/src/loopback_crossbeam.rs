// use dasp::sample::ToSample;
// use vosk::{DecodingState, Recognizer};
// use crate::PcmCallback;
//
// fn handle_input_data<T, U>(
//     input: &[T],
//     writer: &WavWriterHandle,
//     pcm_callback: Option<&PcmCallback>,
//     only_pcm: bool,
//     recognizer: &mut MutexGuard<Recognizer>,
// ) where
//     T: Sample + ToSample<i16> + ToSample<f32>,
//     U: Sample + hound::Sample + FromSample<T>,
// {
//     if only_pcm {
//         if let Some(callback) = pcm_callback {
//             let pcm_data: Vec<i16> = input.iter().map(|&x| x.to_sample::<i16>()).collect();
//             let state = recognizer.accept_waveform(pcm_data.as_slice()).unwrap();
//             match state {
//                 DecodingState::Running => {
//                     let partial = recognizer.partial_result().partial;
//                     if !partial.is_empty() {
//                         println!("Partial result: {partial}");
//                         callback(partial);
//                     }
//                 }
//                 DecodingState::Finalized => {
//                     let result = recognizer.result().multiple().unwrap();
//                     println!("complete result :{:?}", result.alternatives[0].text)
//                 }
//                 DecodingState::Failed => eprintln!("Failed to decode the audio by vosk model"),
//             }
//         }
//     } else if let Ok(mut guard) = writer.try_lock() {
//         if let Some(writer) = guard.as_mut() {
//             for &sample in input.iter() {
//                 let sample: U = U::from_sample(sample);
//                 writer.write_sample(sample).ok();
//             }
//         }
//     }
// }
