import tkinter as tk
from tkinter import filedialog, messagebox
import shlex
import os

OPCODE_TABLE = {
    "RESERVED": 0x00, "PRINT": 0x01, "LOAD_RAM": 0x02, "READ_FILE": 0x03,
    "ASK_INT": 0x04, "ASK_STRING": 0x05, "WAIT_INT": 0x06, "WAIT_STRING": 0x07,
    "LOAD_STD": 0x08, "ADD": 0x09, "SUB": 0x10, "MUL": 0x11, "DIV": 0x12,
    "CHECK_HW": 0x13, "WRITE_FILE": 0x14, "CREATE_FILE": 0x15, "SHUTDOWN": 0x16,
    "RESTART": 0x17, "BIOS_REBOOT": 0x18, "BOOT_LEGACY": 0x19, "BOOT_ALT": 0x20,
    "COMPILE": 0x21, "TERMINAL": 0x22, "MAINTAIN": 0x23, "FETCH_NET": 0x24,
    "UPDATE_NET": 0x25, "CHECK_PARTITIONS": 0x26, "CONNECT_NET": 0x27,
    "BOOT_MENU": 0x28, "CLOCK": 0x29, "UNLOAD_RAM": 0x30, "BREAK": 0x31,
    "STORE_VAR": 0x32, "HALT": 0xFF,
}

def first_pass(lines):
    labels = {}
    pc = 0
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.endswith(":"):
            labels[line[:-1]] = pc
        else:
            parts = shlex.split(line)
            if not parts:
                continue
            instr = parts[0].upper()
            if instr == "PRINT":
                pc += 2 + len(parts[1])
            elif instr == "HALT":
                pc += 3
            else:
                pc += 1
    return labels

def second_pass(lines, labels):
    bytecode = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#") or line.endswith(":"):
            continue
        parts = shlex.split(line)
        if not parts:
            continue
        instr = parts[0].upper()
        opcode = OPCODE_TABLE.get(instr)
        if opcode is None:
            raise ValueError(f"Unknown instruction: {instr}")
        bytecode.append(opcode)
        if instr == "PRINT":
            string = parts[1]
            bytecode.append(len(string))
            bytecode.extend(ord(c) for c in string)
        elif instr == "HALT":
            label = parts[1]
            offset = labels.get(label)
            if offset is None:
                raise ValueError(f"Undefined label: {label}")
            bytecode.extend(offset.to_bytes(2, "little"))
    return bytecode

def assemble(text):
    lines = text.splitlines()
    labels = first_pass(lines)
    return second_pass(lines, labels)

def build_gui():
    root = tk.Tk()
    root.title("ARC Bytecode Assembler")
    root.geometry("800x600")

    editor = tk.Text(root, wrap="word", font=("Courier", 12))
    editor.pack(fill="both", expand=True)

    output = tk.Text(root, wrap="none", font=("Courier", 10), height=6, bg="#f0f0f0")
    output.pack(fill="x")

    def assemble_and_show():
        code = editor.get("1.0", "end-1c")
        output.delete("1.0", "end")
        try:
            bytecode = assemble(code)
            hex_dump = " ".join(f"{b:02X}" for b in bytecode)
            output.insert("end", hex_dump)
        except Exception as e:
            output.insert("end", f"❌ Error: {e}")

    def save_to_file():
        try:
            bytecode = assemble(editor.get("1.0", "end-1c"))
            path = filedialog.asksaveasfilename(defaultextension=".arc", filetypes=[("ARC Bytecode", "*.arc")])
            if path:
                with open(path, "wb") as f:
                    f.write(bytearray(bytecode))
                messagebox.showinfo("Saved", f"Bytecode saved to {path}")
        except Exception as e:
            messagebox.showerror("Error", str(e))

    def handle_drop(event):
        path = event.data.strip("{}")
        if os.path.isfile(path) and path.endswith(".txt"):
            with open(path, "r") as f:
                content = f.read()
            editor.delete("1.0", "end")
            editor.insert("1.0", content)
            assemble_and_show()

    # Drag-and-drop support
    try:
        import tkinterdnd2
        root.destroy()
        root = tkinterdnd2.TkinterDnD.Tk()
        root.title("ARC Bytecode Assembler")
        root.geometry("800x600")

        editor = tk.Text(root, wrap="word", font=("Courier", 12))
        editor.pack(fill="both", expand=True)
        editor.drop_target_register(tkinterdnd2.DND_FILES)
        editor.dnd_bind("<<Drop>>", handle_drop)

        output = tk.Text(root, wrap="none", font=("Courier", 10), height=6, bg="#f0f0f0")
        output.pack(fill="x")

        btn_frame = tk.Frame(root)
        btn_frame.pack(fill="x")
        tk.Button(btn_frame, text="Assemble", command=assemble_and_show).pack(side="left", padx=5, pady=5)
        tk.Button(btn_frame, text="Save .arc", command=save_to_file).pack(side="left", padx=5)
    except ImportError:
        messagebox.showwarning("Drag-and-Drop Disabled", "Install tkinterdnd2 for drag-and-drop support:\n\npip install tkinterdnd2")

    root.mainloop()

if __name__ == "__main__":
    build_gui()