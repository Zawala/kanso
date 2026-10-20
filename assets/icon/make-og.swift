// Draws the link-preview image every published note shares (1200 x 630): the
// icon's ink ensō and vermilion seal on warm paper, beside the wordmark.
// Usage: swift make-og.swift <out.png>   (cloud/public/og/kanso.png)
import AppKit

let out = CommandLine.arguments[1]
let width = 1200.0, height = 630.0

let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: Int(width), pixelsHigh: Int(height),
                           bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                           colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
let cg = NSGraphicsContext.current!.cgContext

func rgb(_ hex: UInt32, _ a: Double = 1) -> CGColor {
    CGColor(red: Double((hex >> 16) & 0xff) / 255, green: Double((hex >> 8) & 0xff) / 255,
            blue: Double(hex & 0xff) / 255, alpha: a)
}
func nsrgb(_ hex: UInt32) -> NSColor { NSColor(cgColor: rgb(hex))! }

// Paper: the icon tile's gradient, top to bottom.
let paper = CGGradient(colorsSpace: CGColorSpaceCreateDeviceRGB(),
                       colors: [rgb(0xFCFAF4), rgb(0xECE5D7)] as CFArray, locations: [0, 1])!
cg.drawLinearGradient(paper, start: CGPoint(x: 0, y: height), end: CGPoint(x: 0, y: 0), options: [])

// The ensō, as in make-icon.swift, at 0.72 of the icon's size.
let centre = CGPoint(x: 330, y: 315), radius = 164.0
let start = 58.0, sweep = 318.0, steps = 240
cg.setStrokeColor(rgb(0x2B2722)); cg.setLineCap(.round)
for i in 0..<steps {
    let t0 = Double(i) / Double(steps), t1 = Double(i + 1) / Double(steps)
    let a0 = (start + sweep * t0) * .pi / 180, a1 = (start + sweep * t1) * .pi / 180
    cg.setLineWidth(0.72 * (30 + 34 * sin(.pi * pow(t0, 0.8))))
    cg.move(to: CGPoint(x: centre.x + radius * cos(a0), y: centre.y + radius * sin(a0)))
    cg.addLine(to: CGPoint(x: centre.x + radius * cos(a1), y: centre.y + radius * sin(a1)))
    cg.strokePath()
}
// The seal, in the gap.
cg.setFillColor(rgb(0xC2492E))
let seal = CGPoint(x: centre.x + 0.72 * 195, y: centre.y + 0.72 * 185)
cg.fillEllipse(in: CGRect(x: seal.x - 17, y: seal.y - 17, width: 34, height: 34))

// The words, left-aligned beside the ensō.
func draw(_ text: String, size: Double, weight: NSFont.Weight, color: UInt32, kern: Double, at point: NSPoint) {
    NSAttributedString(string: text, attributes: [
        .font: NSFont.systemFont(ofSize: size, weight: weight),
        .foregroundColor: nsrgb(color),
        .kern: kern,
    ]).draw(at: point)
}
let left = 600.0
draw("Kanso", size: 112, weight: .semibold, color: 0x2B2722, kern: -2, at: NSPoint(x: left - 6, y: 318))
cg.setFillColor(rgb(0xC2492E)); cg.fill(CGRect(x: left, y: 300, width: 56, height: 4))
draw("A note, shared with you.", size: 38, weight: .regular, color: 0x2B2722, kern: 0, at: NSPoint(x: left, y: 228))
draw("End to end encrypted. Opens in your browser.", size: 24, weight: .regular, color: 0x76736C, kern: 0.2,
     at: NSPoint(x: left, y: 186))

NSGraphicsContext.restoreGraphicsState()
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: out))
