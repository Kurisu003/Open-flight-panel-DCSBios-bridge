import re

in_path, out_path = "custom_font.txt", "output.txt"
with open(in_path, newline="") as f, open(out_path, "w", newline="") as g:
    g.writelines(l if l in ("\n", "\r\n") else l[6:] for l in f)



in_path, out_path = "output.txt", "output2.txt"

with open(in_path, newline="") as fin, open(out_path, "w", newline="") as fout:
    for line in fin:
        if not line.strip():           # keep empty lines (including CRLF vs LF)
            fout.write(line)
            continue
        m = re.match(r'\s*((?:[0-9A-Fa-f]{2}\s+)*[0-9A-Fa-f]{2})', line)
        if m:
            fout.write(m.group(1).strip() + "\n")

in_path, out_path = "output2.txt", "output3.txt"
with open(in_path, "r", encoding="utf-8", newline="") as f:
    s = f.read()

s = s.replace("\r\n", "\n").replace("\r", "\n")      # normalize line breaks
s = s.replace(" ", "")                               # drop spaces
s = re.sub(r'(?<!\n)\n(?!\n)', "", s)                # remove single newlines
s = re.sub(r'\n{2,}', "\n\n", s)                     # keep doubles as doubles

with open(out_path, "w", encoding="utf-8", newline="") as g:
    g.write(s)

in_path, out_path = "output3.txt", "output4.txt"

with open(in_path, newline="") as f, open(out_path, "w", newline="") as g:
    for l in f:
        # if l.endswith("\r\n"):
        #     g.write(l[:-2][54:] + "\r\n")
        # elif l.endswith("\n"):
        #     g.write(l[:-1][54:] + "\n")
        # elif l.endswith("\r"):
        #     g.write(l[:-1][54:] + "\r")
        # else:
            g.write(l[54:])