// Draws Kanso's mobile app icons from the same ensō and seal as the desktop
// icon (make-icon.swift), laid out for each platform's mask.
// Usage: swift make-mobile-icon.swift <mode> <out.png> [size]
//   ios         opaque full-bleed square; iOS rounds the corners itself
//   background  Android adaptive background: the tile's paper gradient
//   foreground  Android adaptive foreground: ensō and seal on transparency,
//               inside the 72 dp visible part of the 108 dp canvas
//   monochrome  Android 13 themed icon: the foreground's shape in one ink
import AppKit
import ImageIO
import UniformTypeIdentifiers

let mode = CommandLine.arguments[1]
let out = CommandLine.arguments[2]
let size = CommandLine.arguments.count > 3 ? Double(CommandLine.arguments[3])! : 1024
let s = size / 1024  // everything below is laid out on the 1024 grid

let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: Int(size), pixelsHigh: Int(size),
                           bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true,
                           isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0,
                           bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
let cg = NSGraphicsContext.current!.cgContext
cg.scaleBy(x: s, y: s)

func rgb(_ hex: UInt32, _ a: Double = 1) -> CGColor {
    CGColor(red: Double((hex >> 16) & 0xff) / 255, green: Double((hex >> 8) & 0xff) / 255,
            blue: Double(hex & 0xff) / 255, alpha: a)
}

// The desktop tile's paper: light at the top, warmer at the foot.
func paper() {
    let gradient = CGGradient(colorsSpace: CGColorSpaceCreateDeviceRGB(),
                              colors: [rgb(0xFCFAF4), rgb(0xECE5D7)] as CFArray, locations: [0, 1])!
    cg.drawLinearGradient(gradient, start: CGPoint(x: 512, y: 1024), end: CGPoint(x: 512, y: 0),
                          options: [.drawsBeforeStartLocation, .drawsAfterEndLocation])
}

// The desktop's 824 px tile grown to `tile` px, so the mark keeps its
// proportions to the visible shape on every platform.
func scaleTile(to tile: Double) {
    let k = tile / 824
    cg.translateBy(x: 512, y: 512)
    cg.scaleBy(x: k, y: k)
    cg.translateBy(x: -512, y: -512)
}

// The ensō and the seal, exactly as make-icon.swift draws them.
func mark(ink: CGColor, seal: CGColor) {
    let centre = CGPoint(x: 505, y: 505), radius = 228.0
    let start = 58.0, sweep = 318.0, steps = 240
    cg.setStrokeColor(ink); cg.setLineCap(.round)
    for i in 0..<steps {
        let t0 = Double(i) / Double(steps), t1 = Double(i + 1) / Double(steps)
        let a0 = (start + sweep * t0) * .pi / 180, a1 = (start + sweep * t1) * .pi / 180
        cg.setLineWidth(30 + 34 * sin(.pi * pow(t0, 0.8)))
        cg.move(to: CGPoint(x: centre.x + radius * cos(a0), y: centre.y + radius * sin(a0)))
        cg.addLine(to: CGPoint(x: centre.x + radius * cos(a1), y: centre.y + radius * sin(a1)))
        cg.strokePath()
    }
    cg.setFillColor(seal)
    cg.fillEllipse(in: CGRect(x: 700 - 24, y: 690 - 24, width: 48, height: 48))
}

switch mode {
case "ios":
    paper()
    scaleTile(to: 1024)
    mark(ink: rgb(0x2B2722), seal: rgb(0xC2492E))
case "background":
    paper()
case "foreground":
    scaleTile(to: 1024 * 72 / 108)
    mark(ink: rgb(0x2B2722), seal: rgb(0xC2492E))
case "monochrome":
    scaleTile(to: 1024 * 72 / 108)
    mark(ink: rgb(0x000000), seal: rgb(0x000000))
default:
    fatalError("unknown mode \(mode)")
}

NSGraphicsContext.restoreGraphicsState()
var image = rep.cgImage!
if mode == "ios" {
    // App Store icons must have no alpha channel at all.
    let opaque = CGContext(data: nil, width: Int(size), height: Int(size), bitsPerComponent: 8,
                           bytesPerRow: 0, space: CGColorSpaceCreateDeviceRGB(),
                           bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue)!
    opaque.draw(image, in: CGRect(x: 0, y: 0, width: size, height: size))
    image = opaque.makeImage()!
}
let dest = CGImageDestinationCreateWithURL(URL(fileURLWithPath: out) as CFURL,
                                           UTType.png.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dest, image, nil)
CGImageDestinationFinalize(dest)
