content = open("rethrow_opt.ll").read()
# find landingpad block
import re
match = re.search(r'(bb\d+:\s*;\s*preds = [^\n]+\n\s*%lp = landingpad[^{]+{[^}]+}\s*cleanup\s*catch ptr null[^\n]+(?:\n[^\n]+){10})', content)
if match:
    print(match.group(1))
