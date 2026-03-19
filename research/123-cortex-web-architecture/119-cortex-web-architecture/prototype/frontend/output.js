var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __typeError = (msg) => {
  throw TypeError(msg);
};
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i = decorators.length - 1, decorator; i >= 0; i--)
    if (decorator = decorators[i])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp(target, key, result);
  return result;
};
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
var __accessCheck = (obj, member, msg) => member.has(obj) || __typeError("Cannot " + msg);
var __privateGet = (obj, member, getter) => (__accessCheck(obj, member, "read from private field"), getter ? getter.call(obj) : member.get(obj));
var __privateAdd = (obj, member, value) => member.has(obj) ? __typeError("Cannot add the same private member more than once") : member instanceof WeakSet ? member.add(obj) : member.set(obj, value);
var _processor;
import { SignalWatcher } from "/node_modules/.vite/deps/@lit-labs_signals.js?v=e2353975";
import { LitElement, html, css } from "/node_modules/.vite/deps/lit.js?v=e2353975";
import { customElement } from "/node_modules/.vite/deps/lit_decorators__js.js?v=e2353975";
import { v0_8 } from "/@fs/Users/xaoj/ICP/nostra/A2UI/renderers/lit/dist/src/index.js";
import "/@fs/Users/xaoj/ICP/nostra/A2UI/renderers/lit/dist/src/0.8/ui/ui.js";
export let A2uiReactWrapper = class extends SignalWatcher(LitElement) {
  constructor() {
    super(...arguments);
    __privateAdd(this, _processor, v0_8.Data.createSignalA2uiMessageProcessor());
  }
  // Provide a method that React can call via a ref to push new messages
  processMessages(messages) {
    __privateGet(this, _processor).processMessages(messages);
  }
  render() {
    const surfaces = __privateGet(this, _processor).getSurfaces();
    if (surfaces.size === 0) {
      return html`<div>Awaiting UI stream...</div>`;
    }
    return html`<section id="surfaces">
      ${Array.from(surfaces.entries()).map(
      ([surfaceId, surface]) => {
        return html`<a2ui-surface
            @a2uiaction=${async (evt) => {
          const [target] = evt.composedPath();
          if (!(target instanceof HTMLElement)) return;
          const context = {};
          if (evt.detail.action.context) {
            for (const item of evt.detail.action.context) {
              if (item.value.literalString) {
                context[item.key] = item.value.literalString;
              }
            }
          }
          const actionPayload = {
            surfaceId,
            name: evt.detail.action.name,
            sourceComponentId: target.id,
            timestamp: (/* @__PURE__ */ new Date()).toISOString(),
            context
          };
          this.dispatchEvent(new CustomEvent("user-action", {
            detail: actionPayload,
            bubbles: true,
            composed: true
          }));
        }}
            .surfaceId=${surfaceId}
            .surface=${surface}
            .processor=${__privateGet(this, _processor)}
          ></a2ui-surface>`;
      }
    )}
    </section>`;
  }
};
_processor = new WeakMap();
__publicField(A2uiReactWrapper, "styles", css`
    :host {
      display: block;
      width: 100%;
      height: 100%;
    }
    #surfaces {
      display: flex;
      flex-direction: column;
      width: 100%;
    }
  `);
A2uiReactWrapper = __decorateClass([
  customElement("a2ui-react-wrapper")
], A2uiReactWrapper);

//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbImEydWktd3JhcHBlci50cyJdLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBTaWduYWxXYXRjaGVyIH0gZnJvbSBcIkBsaXQtbGFicy9zaWduYWxzXCI7XG5pbXBvcnQgeyBMaXRFbGVtZW50LCBodG1sLCBjc3MgfSBmcm9tIFwibGl0XCI7XG5pbXBvcnQgeyBjdXN0b21FbGVtZW50IH0gZnJvbSBcImxpdC9kZWNvcmF0b3JzLmpzXCI7XG5pbXBvcnQgeyB2MF84IH0gZnJvbSBcIkBhMnVpL2xpdFwiO1xuaW1wb3J0IFwiQGEydWkvbGl0L3VpXCI7XG5cbkBjdXN0b21FbGVtZW50KFwiYTJ1aS1yZWFjdC13cmFwcGVyXCIpXG5leHBvcnQgY2xhc3MgQTJ1aVJlYWN0V3JhcHBlciBleHRlbmRzIFNpZ25hbFdhdGNoZXIoTGl0RWxlbWVudCkge1xuICAgICNwcm9jZXNzb3IgPSB2MF84LkRhdGEuY3JlYXRlU2lnbmFsQTJ1aU1lc3NhZ2VQcm9jZXNzb3IoKTtcblxuICAgIHN0YXRpYyBzdHlsZXMgPSBjc3NgXG4gICAgOmhvc3Qge1xuICAgICAgZGlzcGxheTogYmxvY2s7XG4gICAgICB3aWR0aDogMTAwJTtcbiAgICAgIGhlaWdodDogMTAwJTtcbiAgICB9XG4gICAgI3N1cmZhY2VzIHtcbiAgICAgIGRpc3BsYXk6IGZsZXg7XG4gICAgICBmbGV4LWRpcmVjdGlvbjogY29sdW1uO1xuICAgICAgd2lkdGg6IDEwMCU7XG4gICAgfVxuICBgO1xuXG4gICAgLy8gUHJvdmlkZSBhIG1ldGhvZCB0aGF0IFJlYWN0IGNhbiBjYWxsIHZpYSBhIHJlZiB0byBwdXNoIG5ldyBtZXNzYWdlc1xuICAgIHB1YmxpYyBwcm9jZXNzTWVzc2FnZXMobWVzc2FnZXM6IGFueVtdKSB7XG4gICAgICAgIHRoaXMuI3Byb2Nlc3Nvci5wcm9jZXNzTWVzc2FnZXMobWVzc2FnZXMpO1xuICAgIH1cblxuICAgIHJlbmRlcigpIHtcbiAgICAgICAgY29uc3Qgc3VyZmFjZXMgPSB0aGlzLiNwcm9jZXNzb3IuZ2V0U3VyZmFjZXMoKTtcbiAgICAgICAgaWYgKHN1cmZhY2VzLnNpemUgPT09IDApIHtcbiAgICAgICAgICAgIHJldHVybiBodG1sYDxkaXY+QXdhaXRpbmcgVUkgc3RyZWFtLi4uPC9kaXY+YDtcbiAgICAgICAgfVxuXG4gICAgICAgIHJldHVybiBodG1sYDxzZWN0aW9uIGlkPVwic3VyZmFjZXNcIj5cbiAgICAgICR7QXJyYXkuZnJvbShzdXJmYWNlcy5lbnRyaWVzKCkpLm1hcChcbiAgICAgICAgICAgIChbc3VyZmFjZUlkLCBzdXJmYWNlXTogYW55KSA9PiB7XG4gICAgICAgICAgICAgICAgcmV0dXJuIGh0bWxgPGEydWktc3VyZmFjZVxuICAgICAgICAgICAgQGEydWlhY3Rpb249JHthc3luYyAoZXZ0OiBhbnkpID0+IHtcbiAgICAgICAgICAgICAgICAgICAgICAgIGNvbnN0IFt0YXJnZXRdID0gZXZ0LmNvbXBvc2VkUGF0aCgpO1xuICAgICAgICAgICAgICAgICAgICAgICAgaWYgKCEodGFyZ2V0IGluc3RhbmNlb2YgSFRNTEVsZW1lbnQpKSByZXR1cm47XG5cbiAgICAgICAgICAgICAgICAgICAgICAgIGNvbnN0IGNvbnRleHQ6IGFueSA9IHt9O1xuICAgICAgICAgICAgICAgICAgICAgICAgaWYgKGV2dC5kZXRhaWwuYWN0aW9uLmNvbnRleHQpIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICBmb3IgKGNvbnN0IGl0ZW0gb2YgZXZ0LmRldGFpbC5hY3Rpb24uY29udGV4dCkge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBpZiAoaXRlbS52YWx1ZS5saXRlcmFsU3RyaW5nKSB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBjb250ZXh0W2l0ZW0ua2V5XSA9IGl0ZW0udmFsdWUubGl0ZXJhbFN0cmluZztcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgIH1cbiAgICAgICAgICAgICAgICAgICAgICAgIH1cblxuICAgICAgICAgICAgICAgICAgICAgICAgY29uc3QgYWN0aW9uUGF5bG9hZCA9IHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICBzdXJmYWNlSWQ6IHN1cmZhY2VJZCxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICBuYW1lOiBldnQuZGV0YWlsLmFjdGlvbi5uYW1lLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgIHNvdXJjZUNvbXBvbmVudElkOiB0YXJnZXQuaWQsXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgdGltZXN0YW1wOiBuZXcgRGF0ZSgpLnRvSVNPU3RyaW5nKCksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgY29udGV4dCxcbiAgICAgICAgICAgICAgICAgICAgICAgIH07XG5cbiAgICAgICAgICAgICAgICAgICAgICAgIHRoaXMuZGlzcGF0Y2hFdmVudChuZXcgQ3VzdG9tRXZlbnQoXCJ1c2VyLWFjdGlvblwiLCB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgZGV0YWlsOiBhY3Rpb25QYXlsb2FkLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgIGJ1YmJsZXM6IHRydWUsXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgY29tcG9zZWQ6IHRydWVcbiAgICAgICAgICAgICAgICAgICAgICAgIH0pKTtcbiAgICAgICAgICAgICAgICAgICAgfX1cbiAgICAgICAgICAgIC5zdXJmYWNlSWQ9JHtzdXJmYWNlSWR9XG4gICAgICAgICAgICAuc3VyZmFjZT0ke3N1cmZhY2V9XG4gICAgICAgICAgICAucHJvY2Vzc29yPSR7dGhpcy4jcHJvY2Vzc29yfVxuICAgICAgICAgID48L2EydWktc3VyZmFjZT5gO1xuICAgICAgICAgICAgfVxuICAgICAgICApfVxuICAgIDwvc2VjdGlvbj5gO1xuICAgIH1cbn1cbiJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7QUFBQSxTQUFTLHFCQUFxQjtBQUM5QixTQUFTLFlBQVksTUFBTSxXQUFXO0FBQ3RDLFNBQVMscUJBQXFCO0FBQzlCLFNBQVMsWUFBWTtBQUNyQixPQUFPO0FBR0EsV0FBTSxtQkFBTixjQUErQixjQUFjLFVBQVUsRUFBRTtBQUFBLEVBQXpEO0FBQUE7QUFDSCxtQ0FBYSxLQUFLLEtBQUssaUNBQWlDO0FBQUE7QUFBQTtBQUFBLEVBZ0JqRCxnQkFBZ0IsVUFBaUI7QUFDcEMsdUJBQUssWUFBVyxnQkFBZ0IsUUFBUTtBQUFBLEVBQzVDO0FBQUEsRUFFQSxTQUFTO0FBQ0wsVUFBTSxXQUFXLG1CQUFLLFlBQVcsWUFBWTtBQUM3QyxRQUFJLFNBQVMsU0FBUyxHQUFHO0FBQ3JCLGFBQU87QUFBQSxJQUNYO0FBRUEsV0FBTztBQUFBLFFBQ1AsTUFBTSxLQUFLLFNBQVMsUUFBUSxDQUFDLEVBQUU7QUFBQSxNQUMzQixDQUFDLENBQUMsV0FBVyxPQUFPLE1BQVc7QUFDM0IsZUFBTztBQUFBLDBCQUNHLE9BQU8sUUFBYTtBQUN0QixnQkFBTSxDQUFDLE1BQU0sSUFBSSxJQUFJLGFBQWE7QUFDbEMsY0FBSSxFQUFFLGtCQUFrQixhQUFjO0FBRXRDLGdCQUFNLFVBQWUsQ0FBQztBQUN0QixjQUFJLElBQUksT0FBTyxPQUFPLFNBQVM7QUFDM0IsdUJBQVcsUUFBUSxJQUFJLE9BQU8sT0FBTyxTQUFTO0FBQzFDLGtCQUFJLEtBQUssTUFBTSxlQUFlO0FBQzFCLHdCQUFRLEtBQUssR0FBRyxJQUFJLEtBQUssTUFBTTtBQUFBLGNBQ25DO0FBQUEsWUFDSjtBQUFBLFVBQ0o7QUFFQSxnQkFBTSxnQkFBZ0I7QUFBQSxZQUNsQjtBQUFBLFlBQ0EsTUFBTSxJQUFJLE9BQU8sT0FBTztBQUFBLFlBQ3hCLG1CQUFtQixPQUFPO0FBQUEsWUFDMUIsWUFBVyxvQkFBSSxLQUFLLEdBQUUsWUFBWTtBQUFBLFlBQ2xDO0FBQUEsVUFDSjtBQUVBLGVBQUssY0FBYyxJQUFJLFlBQVksZUFBZTtBQUFBLFlBQzlDLFFBQVE7QUFBQSxZQUNSLFNBQVM7QUFBQSxZQUNULFVBQVU7QUFBQSxVQUNkLENBQUMsQ0FBQztBQUFBLFFBQ04sQ0FBQztBQUFBLHlCQUNJLFNBQVM7QUFBQSx1QkFDWCxPQUFPO0FBQUEseUJBQ0wsbUJBQUssV0FBVTtBQUFBO0FBQUEsTUFFNUI7QUFBQSxJQUNKLENBQUM7QUFBQTtBQUFBLEVBRUw7QUFDSjtBQWpFSTtBQUVBLGNBSFMsa0JBR0YsVUFBUztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFIUCxtQkFBTjtBQUFBLEVBRE4sY0FBYyxvQkFBb0I7QUFBQSxHQUN0QjsiLCJuYW1lcyI6W119