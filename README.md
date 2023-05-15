# A WAV Player written in Rust, using WASAPI (Windows Audio Session Applicable Programming Interface)

This is a basic Wav playing program that is written in Rust, using the WASAPI, like title above. <br/>It is protected against signal interrupts so it can exit safely. <br/>This code mostly consists of unsafe Rust code, however it is developed with RAII idioms. <br/><br/>Has a very simplistic usage:
<br/>
<br/>
<code>./wav_player YOUR_WAV_FILE_NAME.wav <-pb> </code>
<br/>
<br/>
<em>The </em><strong>-pb</strong> <em>flag stands for "playback", so should you use this flag, your input wav file will be playing continously.</em>
<br/>
<br/>
<footer>Author: Fırat Özkan</footer>
