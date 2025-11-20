#!/bin/bash
# Check if Ansible vault files are properly encrypted
# Vault files should start with $ANSIBLE_VAULT

set -e

EXIT_CODE=0

echo "Checking Ansible vault files for encryption..."

for file in "$@"; do
    # Check if file exists and is not empty
    if [ ! -f "$file" ]; then
        continue
    fi
    
    if [ ! -s "$file" ]; then
        echo "⚠️  Warning: $file is empty"
        continue
    fi
    
    # Check if file starts with $ANSIBLE_VAULT
    if head -n 1 "$file" | grep -q '^\$ANSIBLE_VAULT'; then
        echo "✓ $file is properly encrypted"
    else
        echo "✗ ERROR: $file is NOT encrypted with Ansible Vault!"
        echo "  Please encrypt this file using: ansible-vault encrypt $file"
        EXIT_CODE=1
    fi
done

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "❌ One or more Ansible vault files are not encrypted!"
    echo "   This could expose sensitive data."
    echo ""
    echo "To fix this, encrypt the files with:"
    echo "  ansible-vault encrypt <filename>"
    echo ""
fi

exit $EXIT_CODE
