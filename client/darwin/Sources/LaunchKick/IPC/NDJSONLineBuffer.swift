import Foundation

enum NDJSONLine: Equatable {
    case line(String)
    case invalidUTF8(Data)
}

struct NDJSONLineBuffer {
    private var buffer = Data()

    mutating func append(_ data: Data) -> [NDJSONLine] {
        buffer.append(data)
        var lines: [NDJSONLine] = []

        while let newlineIndex = buffer.firstIndex(of: 0x0A) {
            let lineData = Data(buffer[..<newlineIndex])
            buffer.removeSubrange(...newlineIndex)

            if let line = String(data: lineData, encoding: .utf8) {
                lines.append(.line(line))
            } else {
                lines.append(.invalidUTF8(lineData))
            }
        }

        return lines
    }
}
