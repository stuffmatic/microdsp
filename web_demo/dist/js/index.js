const onMicAccessButtonClick = (e) => {
  const options = {
    workletNodeOptions: {
      numberOfInputs: 1,
      numberOfOutputs: 0,
      outputChannelCount: []
    },
    // The following should be defined in demo specific js
    workletNodeName,
    workletProcessorUrl,
    wasmUrl,
  }

  startAudioWorklet(options).then((workletNode) => {
    console.log("Audio worklet node started.")
    document.getElementById("mic-access-modal").style = "display: none";
    onWorkletNodeCreated(workletNode)
  }).catch((error) => {
    console.log("Failed to start audio worklet node. " + error)
    document.getElementById("mic-access-prompt").style = "display: none;"
    document.getElementById("mic-access-error-message").innerText = error
  })
}

/*
name, min, max, default
*/
const registerControls = (controls) => {

}