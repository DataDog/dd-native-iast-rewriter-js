/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
import * as fs from 'fs'
import * as path from 'path'

const classifiers = [
    {src: 'linux-x64-gnu', dst: 'linux-x64/node.napi.glibc.node'},
    {src: 'linux-x64-musl', dst: 'linux-x64/node.napi.musl.node'},
    {src: 'win32-x64-msvc', dst: 'win32-x64/node.napi.node'},
    {src: 'darwin-x64', dst: 'darwin-x64/node.napi.node'},
    {src: 'darwin-arm64', dst: 'darwin-arm64/node.napi.node'},
]

const copyFile = async (src, dest) => {
    return fs.promises.access(src, fs.constants.F_OK).then(
        () => fs.promises.copyFile(src, dest),
        (_) => Promise.resolve(),
    )
}

const copyArtifact = (classifier) => {
    const filename = `iast-rewriter.${classifier.src}.node`
    let sourceNode = path.join(process.cwd(), filename)
    if (!fs.existsSync(sourceNode)){
        sourceNode = path.join(process.cwd(), `iast-rewriter.${classifier.src}`, filename)
    }
    const destNode = path.join(`${process.cwd()}`, 'prebuilds', classifier.dst)
    return copyFile(sourceNode, destNode)
}

const prepare = () => {
    return Promise.all(classifiers.map(copyArtifact))
}

await prepare()
