;; Raylib Basic Window Example in Lisp
;; This demonstrates using FFI to call the Raylib game library
;;
;; Prerequisites:
;;   - Install raylib: https://github.com/raysan5/raylib
;;   - Make sure libraylib.so (or .dylib) is in your library path
;;
;; Run:
;;   ./target/release/bytecomp examples/raylib-example/raylib-window.lisp -o raylib-window.bc
;;   ./target/release/lisp-vm raylib-window.bc

;; ============================================================
;; Color struct {r, g, b, a} packed as little-endian uint32:
;;   value = r + (g * 256) + (b * 65536) + (a * 16777216)
;; ============================================================

;; RAYWHITE = {245, 245, 245, 255}
(def COLOR_RAYWHITE 4294309365)

;; LIGHTGRAY = {200, 200, 200, 255}
(def COLOR_LIGHTGRAY 4291348680)

;; BLACK = {0, 0, 0, 255}
(def COLOR_BLACK 4278190080)

;; WHITE = {255, 255, 255, 255}
(def COLOR_WHITE 4294967295)

;; RED = {230, 41, 55, 255}
(def COLOR_RED 4281878758)

;; GREEN = {0, 228, 48, 255}
(def COLOR_GREEN 4281401344)

;; BLUE = {0, 121, 241, 255}
(def COLOR_BLUE 4294016256)

;; ============================================================
;; Load Raylib Library
;; ============================================================

;; Try to load raylib (different names on different systems)
(def raylib
  (ffi-load "libraylib.so"))

;; ============================================================
;; Get Raylib Function Pointers
;; ============================================================

;; void InitWindow(int width, int height, const char *title)
(def InitWindow-ptr (ffi-symbol raylib "InitWindow"))

;; void CloseWindow(void)
(def CloseWindow-ptr (ffi-symbol raylib "CloseWindow"))

;; bool WindowShouldClose(void) - returns int (0 = false, non-zero = true)
(def WindowShouldClose-ptr (ffi-symbol raylib "WindowShouldClose"))

;; void SetTargetFPS(int fps)
(def SetTargetFPS-ptr (ffi-symbol raylib "SetTargetFPS"))

;; void BeginDrawing(void)
(def BeginDrawing-ptr (ffi-symbol raylib "BeginDrawing"))

;; void EndDrawing(void)
(def EndDrawing-ptr (ffi-symbol raylib "EndDrawing"))

;; void ClearBackground(Color color) - Color is passed as uint32
(def ClearBackground-ptr (ffi-symbol raylib "ClearBackground"))

;; void DrawText(const char *text, int posX, int posY, int fontSize, Color color)
(def DrawText-ptr (ffi-symbol raylib "DrawText"))

;; ============================================================
;; Wrapper Functions for cleaner syntax
;; ============================================================

(defun init-window ((width height title)
  (ffi-call InitWindow-ptr (:int32 :int32 :string) :void width height title)))

(defun close-window (()
  (ffi-call CloseWindow-ptr () :void)))

(defun window-should-close (()
  ;; Returns 0 (false) or non-zero (true)
  (ffi-call WindowShouldClose-ptr () :int32)))

(defun set-target-fps ((fps)
  (ffi-call SetTargetFPS-ptr (:int32) :void fps)))

(defun begin-drawing (()
  (ffi-call BeginDrawing-ptr () :void)))

(defun end-drawing (()
  (ffi-call EndDrawing-ptr () :void)))

(defun clear-background ((color)
  (ffi-call ClearBackground-ptr (:uint32) :void color)))

(defun draw-text ((text x y font-size color)
  (ffi-call DrawText-ptr (:string :int32 :int32 :int32 :uint32) :void
            text x y font-size color)))

;; ============================================================
;; Main Program
;; ============================================================

(def screen-width 800)
(def screen-height 450)

;; Initialize window
(print "Initializing Raylib window...")
(init-window screen-width screen-height "Lisp + Raylib - Basic Window")

;; Set target FPS
(set-target-fps 60)

(print "Starting game loop... Press ESC or close window to exit.")

;; Game loop - using loop/recur because tail recursion issue
(loop ()
  (if (== (window-should-close) 0)
    (do
      ;; Draw frame
      (begin-drawing)
      (clear-background COLOR_RAYWHITE)
      (draw-text "Let's make Lisp run with Raylib!" 190 200 20 COLOR_LIGHTGRAY)
      (draw-text "This window was created using FFI from Lisp!" 160 230 20 COLOR_LIGHTGRAY)
      (end-drawing)
      ;; Continue loop
      (recur))
    ;; Exit loop - return false to end
    false))

;; Cleanup
(print "Closing window...")
(close-window)
(print "Done!")
