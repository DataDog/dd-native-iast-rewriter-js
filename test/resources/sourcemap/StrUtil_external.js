'use strict'
exports.__esModule = true
var StrUtil_external = /** @class */ (function () {
  function StrUtil() {}
  StrUtil.prototype.sep = function (a) {
    return '-' + a
  }
  StrUtil.prototype.addSep = function (a, b) {
    return a + this.sep(this.toStr(b))
  }
  StrUtil.prototype.toStr = function (a) {
    return a.toString()
  }
  StrUtil.prototype.add = function (a, b) {
    return a + b
  }
  return StrUtil
})()
exports['default'] = StrUtil_external
//# sourceMappingURL=StrUtil.js.map
