import Foundation

struct NDJSONLineBuffer {
    private var buffer = Data()

    mutating func append(_ data: Data) -> [String] {
        buffer.append(data)
        var lines: [String] = []

        while let newlineIndex = buffer.firstIndex(of: 0x0A) {
            let lineData = Data(buffer[..<newlineIndex])
            buffer.removeSubrange(...newlineIndex)

            if let line = String(data: lineData, encoding: .utf8) {
                lines.append(line)
            }
        }

        return lines
    }
}
