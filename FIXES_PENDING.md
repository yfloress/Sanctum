# Pending Fixes — Finances Account Icons

## Bug: Icon not updating after selection (create modal + detail panel)

### Síntoma
- Al crear una cuenta y seleccionar un ícono en el picker, el preview sigue
  mostrando el ícono por defecto (ej. chanchito para savings).
- Al abrir "Change Icon" en el panel de detalle y seleccionar un ícono, el
  panel-acc-icon sigue mostrando el default.

### Causa probable
1. **Create modal**: `pickedIcon` es `$derived` y debería reaccionar a `accIcon`,
   pero si el re-render no ocurre por algún edge case de Svelte 5, el preview
   no se actualiza. Verificar con console.log si `accIcon` cambia correctamente.

2. **Detail panel**: `changeAccountIcon()` llama `fetchAccountDetails()` y
   reasigna `selectedAccount`. Si `selectedAccount.icon_path` sigue siendo `null`
   después de la llamada, el problema está en el backend o en la serialización.
   Verificar que `update_account_icon` persiste y que `fetch_account_details`
   retorna el `icon_path` actualizado.

### Debugging steps
```typescript
// En changeAccountIcon, agregar logs:
console.log('antes:', selectedAccount?.icon_path)
await financeApi.updateAccountIcon(selectedAccount.id, icon)
selectedAccount = await financeApi.fetchAccountDetails(selectedAccount.id)
console.log('después:', selectedAccount?.icon_path)
```

```typescript
// En submitAccount, verificar que newAcc se encuentra:
const newAcc = accountsData?.accounts.find(a => !before.has(a.id))
console.log('newAcc:', newAcc?.id, 'accIcon:', accIcon)
if (newAcc) {
  await financeApi.updateAccountIcon(newAcc.id, accIcon)
  ...
}
```

### Posible fix alternativo (create modal)
Si `$derived` sigue sin actualizarse, forzar re-render con un `$effect`:
```typescript
let pickedIconSrc = $state('')
let pickedIconGeneric = $state(true)
$effect(() => {
  const found = accIcon ? ACCOUNT_ICONS.find(i => i.value === accIcon) : null
  pickedIconSrc = found ? found.src : getDefaultIconPath(accType)
  pickedIconGeneric = found ? found.generic : true
})
```

---

## Fixed in this session
- [x] Account types corrected: bank, savings, credit, cash, other (removed checking/investment)
- [x] `getDefaultIconPath` handles `credit_card` (backend-normalized value)
- [x] Default type changed to `bank`
- [x] `$derived` replaces `{@const}` for `pickedIcon` to ensure reactivity
- [x] `refreshAccounts()` restored in edit account flow
