const editedFilenames = []
function isFileEdited (filename) {
  for (let i = 0; i < editedFilenames.length; i++) {
    if (editedFilenames[i].indexOf(filename) > -1) {
      return true
    }
  }
  return false
}

function addEditedFile (filename) {
  editedFilenames.push(filename)
}
module.exports = {
  isFileEdited,
  addEditedFile
}
