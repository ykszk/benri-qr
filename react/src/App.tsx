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
        const h = xlsx2html(arr, "qr", "ja");
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
      <button title="Restart from the start" onClick={() => { setErrMessage(""); setHtml(""); }}>Reset</button>
    )
    const save_button = (
      <button title="Save output" onClick={() => { downloadAsHtml(html, "qrcode.html") }}>Save</button>
    )
    const print_button = (
      <button title="Print QrCode" onClick={() => {
        const preview = document.getElementById("preview");
        if (preview !== null) {
          const cw = (preview as HTMLIFrameElement).contentWindow;
          if (cw !== null) {
            cw.print();
          }
        }
      }}>Print</button>
    )
    return (
      <div className="App">
        <iframe id="preview" title="qrcode" srcDoc={html}></iframe>
        <div id="buttons">
          {print_button}
          {save_button}
          {reset_button}
        </div>
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
            <p>Drop .xlsx file or click to open.</p>
        }
      </div>
    </div>
  );
}

export default App;
