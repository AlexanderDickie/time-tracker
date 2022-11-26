import { invoke } from "@tauri-apps/api/tauri"

export default function alert() {
    // invoking command from rust instead of calling api/window close()
    // which is not working- possibly because of nexjs ssr?
    function close() {
        return invoke('close_alert_window')
            .then(() => {})
            .catch(() => {})
   }
    return (
        <div>
            <h1>
                Finished
            </h1>
            <button onClick={close}> ok </button>
            <audio
                autoPlay
                src="gong.mp3">
                <a href="gong.mp3">
                    Download audio
                </a>
            </audio>
        </div>
    )
}
