import React, { useEffect, useRef, useState } from 'react'

const CANVAS_WIDTH = 128
const CANVAS_HEIGHT = 64

interface RoboEyesState {
  mood: 'DEFAULT' | 'TIRED' | 'ANGRY' | 'HAPPY'
  eyeX: number
  eyeY: number
  blinking: boolean
  blinkProgress: number
  lookDirection: number
}

export const RoboEyes: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [state, setState] = useState<RoboEyesState>({
    mood: 'DEFAULT',
    eyeX: 0,
    eyeY: 0,
    blinking: false,
    blinkProgress: 0,
    lookDirection: 0,
  })
  const frameRef = useRef(0)
  const blinkTimerRef = useRef(0)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const animate = () => {
      frameRef.current = (frameRef.current + 1) % 18000

      setState((prevState) => {
        let newState = { ...prevState }

        // Blink logic (every ~300 frames = ~5s at 60fps)
        blinkTimerRef.current++
        if (blinkTimerRef.current > 300) {
          if (!prevState.blinking) {
            newState.blinking = true
            newState.blinkProgress = 0
          }
        }

        if (newState.blinking) {
          newState.blinkProgress += 0.1
          if (newState.blinkProgress >= 1) {
            newState.blinking = false
            newState.blinkProgress = 0
            blinkTimerRef.current = 0
          }
        }

        // Eye movement (slow drift)
        newState.lookDirection = Math.sin(frameRef.current / 500) * 0.3
        newState.eyeX = newState.lookDirection
        newState.eyeY = Math.cos(frameRef.current / 700) * 0.2

        return newState
      })
    }

    const animationId = setInterval(animate, 16)
    return () => clearInterval(animationId)
  }, [])

  // Draw loop
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    ctx.fillStyle = 'transparent'
    ctx.clearRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT)

    const eyeLeftX = 32
    const eyeRightX = 96
    const eyeY = 32
    const eyeRadius = 12
    const pupilRadius = 6

    // Draw left eye
    drawEye(ctx, eyeLeftX, eyeY, eyeRadius, pupilRadius, state)

    // Draw right eye
    drawEye(ctx, eyeRightX, eyeY, eyeRadius, pupilRadius, state)
  }, [state])

  return (
    <canvas
      ref={canvasRef}
      width={CANVAS_WIDTH}
      height={CANVAS_HEIGHT}
      style={{ width: '100%', height: '100%' }}
    />
  )
}

function drawEye(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  radius: number,
  pupilRadius: number,
  state: RoboEyesState,
) {
  // Draw sclera (white of eye)
  ctx.fillStyle = '#ffffff'
  ctx.beginPath()
  ctx.arc(x, y, radius, 0, Math.PI * 2)
  ctx.fill()

  // Draw iris
  ctx.fillStyle = '#1b2026'
  const pupilX = x + state.eyeX * (radius - pupilRadius)
  const pupilY = y + state.eyeY * (radius - pupilRadius)

  ctx.beginPath()
  ctx.arc(pupilX, pupilY, pupilRadius, 0, Math.PI * 2)
  ctx.fill()

  // Draw eyelid if blinking
  if (state.blinking) {
    ctx.fillStyle = '#13161c'
    const eyelidHeight = radius * 2 * state.blinkProgress
    ctx.fillRect(x - radius, y - radius, radius * 2, eyelidHeight)
  }
}
