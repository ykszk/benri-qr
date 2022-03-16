import './App.css';
import { useEffect, useState, useCallback } from 'react'
import { useDropzone } from 'react-dropzone'
import init, { xlsx2html } from "benri-qr";
function downloadAsHtml(text: string, name: string) {
  const blob = new Blob([text], { type: 'text/html' });
  var link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = name;
  link.click();
  link.remove();
}
function App() {
  useEffect(() => {
    init().then(() => {
      console.log("wasm loaded.")
    })
  }, [])
  const [html, setHtml] = useState("");
  const [errMessage, setErrMessage] = useState("");

  const onDrop = useCallback(acceptedFiles => {
    const filename = acceptedFiles[0].name;
    console.log(filename);
    const reader = new FileReader();
    reader.readAsArrayBuffer(acceptedFiles[0]);
    reader.onload = function () {
      try {
        const buf = reader.result as ArrayBuffer;
        const arr = new Uint8Array(buf);
        const h = xlsx2html(arr, "qr");
        setHtml(h);
      } catch (error) {
        console.error(error);
        setErrMessage(error as string);
      }
    }
  }, [])

  const { getRootProps, getInputProps, isDragActive } = useDropzone({ onDrop })
  if (html.length !== 0) {
    const reset_button = (
      <button title="Restart from the start" onClick={() => { setHtml("") }}>Reset</button>
    )
    const save_button = (
      <button title="Save output" onClick={() => { downloadAsHtml(html, "qrcode.html") }}>Save</button>
    )
    return (
      <div className="App">
        <iframe title="qrcode" srcDoc={html}></iframe>
        {reset_button}
        {save_button}
      </div>
    )
  }


  const err_if_any = errMessage.length !== 0 ?
    <p className="error">{errMessage}</p> : <></>;
  return (
    <div className="App">
      {err_if_any}
      <div className={"dropzone" + (isDragActive ? " dragActive" : "")} {...getRootProps()}>
        <input {...getInputProps()} />
        {
          isDragActive ?
            <p>Open the file ...</p> :
            <p>Drop .xlsx with Name, Reading, TEL, Email, Memo, Birthday, Address, URL, Nickname columns.
              Any columns but "Name" are optional..</p>
        }
      </div>
    </div>
  );
}

export default App;
