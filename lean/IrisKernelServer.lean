import IrisKernel.FFI
/-!
# IRIS Kernel IPC Server

Minimal stdin/stdout IPC server for the IRIS proof kernel.
Reads requests from stdin, dispatches to the @[export] kernel functions,
writes results back to stdout.

## Wire format

Request:  rule_id (1 byte) + payload_len (4 bytes LE) + payload (payload_len bytes)
Response: result_len (4 bytes LE) + result (result_len bytes)

Special rule_id 0 = cost_leq check (returns 1 byte: 0 or 1)
Special response for rule_id 0: result_len (4 bytes LE) + 1 byte result

Rule IDs 1-20 map to the 20 kernel inference rules.
Rule ID 255 = shutdown (server exits cleanly).
-/

open IrisKernel.FFI

/-- Read exactly `n` bytes from stdin. Returns none on EOF. -/
def readExact (stream : IO.FS.Stream) (n : Nat) : IO (Option ByteArray) := do
  let mut buf := ByteArray.empty
  let mut remaining := n
  while remaining > 0 do
    let chunk ← stream.read (remaining.toUSize)
    if chunk.size == 0 then
      return none  -- EOF
    buf := buf ++ chunk
    remaining := remaining - chunk.size
  return some buf

/-- Read a UInt32 LE from 4 bytes. -/
def readUInt32LE (bytes : ByteArray) : UInt32 :=
  let b0 := bytes.get! 0
  let b1 := bytes.get! 1
  let b2 := bytes.get! 2
  let b3 := bytes.get! 3
  b0.toUInt32 ||| (b1.toUInt32 <<< 8) ||| (b2.toUInt32 <<< 16) ||| (b3.toUInt32 <<< 24)

/-- Write a UInt32 LE to 4 bytes. -/
def writeUInt32LE (v : UInt32) : ByteArray :=
  ByteArray.empty
    |>.push v.toUInt8
    |>.push (v >>> 8).toUInt8
    |>.push (v >>> 16).toUInt8
    |>.push (v >>> 24).toUInt8

/-- Dispatch a rule by ID and return the result ByteArray. -/
partial def dispatchRule (ruleId : UInt8) (payload : ByteArray) : ByteArray :=
  match ruleId.toNat with
  | 0 =>
    -- Special: cost_leq check. Returns single byte.
    let result := checkCostLeqFFI payload
    ByteArray.empty |>.push result
  | 1  => assumeFFI payload
  | 2  => introFFI payload
  | 3  => elimFFI payload
  | 4  => reflFFI payload
  | 5  => symmFFI payload
  | 6  => transFFI payload
  | 7  => congrFFI payload
  | 8  => typeCheckNodeFullFFI payload
  | 9  => costSubsumeFFI payload
  | 10 => costLeqRuleFFI payload
  | 11 => refineIntroFFI payload
  | 12 => refineElimFFI payload
  | 13 => natIndFFI payload
  | 14 => structuralIndFFI payload
  | 15 => letBindFFI payload
  | 16 => matchElimFFI payload
  | 17 => foldRuleFFI payload
  | 18 => typeAbstFFI payload
  | 19 => typeAppFFI payload
  | 20 => guardRuleFFI payload
  | _  => encodeFailure 99  -- unknown rule

/-- Main server loop: read requests, dispatch, write responses. -/
partial def serverLoop (stdin : IO.FS.Stream) (stdout : IO.FS.Stream) : IO Unit := do
  -- Read rule_id (1 byte)
  let some ruleIdBytes ← readExact stdin 1
    | return ()  -- EOF, exit cleanly
  let ruleId := ruleIdBytes.get! 0

  -- Rule 255 = shutdown
  if ruleId == 255 then
    return ()

  -- Read payload_len (4 bytes LE)
  let some lenBytes ← readExact stdin 4
    | return ()  -- EOF
  let payloadLen := readUInt32LE lenBytes

  -- Read payload
  let some payload ← readExact stdin payloadLen.toNat
    | return ()  -- EOF

  -- Dispatch and get result
  let result := dispatchRule ruleId payload

  -- Write response: result_len (4 bytes LE) + result bytes
  let lenOut := writeUInt32LE result.size.toUInt32
  stdout.write lenOut
  stdout.write result
  stdout.flush

  -- Continue loop
  serverLoop stdin stdout

def main : IO Unit := do
  let stdin ← IO.getStdin
  let stdout ← IO.getStdout
  -- Write a ready signal: 4 bytes "IRIS" magic
  stdout.write (ByteArray.empty |>.push 0x49 |>.push 0x52 |>.push 0x49 |>.push 0x53)
  stdout.flush
  serverLoop stdin stdout
