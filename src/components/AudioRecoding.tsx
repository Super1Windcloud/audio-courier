import type React from "react";
import styled from "styled-components";

const Button = () => {
	return (
		<StyledWrapper>
			<div className="container-vao">
				<input
					type="checkbox"
					className="input-orb"
					id="v.a.o."
					name="v.a.o."
					style={{ display: "none" }}
				/>
				<label htmlFor="v.a.o." className="orb">
					<div className="icons">
						<svg
							className="svg"
							xmlns="http://www.w3.org/2000/svg"
							width={24}
							height={24}
							viewBox="0 0 24 24"
						>
							<title>Audio Icons</title>
							<g className="close">
								<path
									fill="currentColor"
									d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59L7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12L5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4"
								/>
							</g>
							<g fill="none" className="mic">
								<rect
									width={8}
									height={13}
									x={8}
									y={2}
									fill="currentColor"
									rx={4}
								/>
								<path
									stroke="currentColor"
									strokeLinecap="round"
									strokeLinejoin="round"
									strokeWidth={2}
									d="M5 11a7 7 0 1 0 14 0m-7 10v-2"
								/>
							</g>
						</svg>
					</div>
					<div className="ball">
						<div className="container-lines" />
						<div className="container-rings" />
					</div>
					<svg style={{ pointerEvents: "none" }}>
						<title>Gooey Filter</title>
						<filter id="gooey">
							{" "}
							<feGaussianBlur in="SourceGraphic" stdDeviation={6} />
							<feColorMatrix
								values="1 0 0 0 0
          0 1 0 0 0
          0 0 1 0 0
          0 0 0 20 -10"
							/>
						</filter>
					</svg>
				</label>
				<div className="container-chat-ia">
					<div className="container-title">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							width={20}
							height={20}
							viewBox="0 0 24 24"
							fill="none"
						>
							<title>Listening Indicator</title>
							<path
								d="M20.5346 6.34625L20.3501 6.7707C20.3213 6.83981 20.2727 6.89885 20.2103 6.94038C20.148 6.98191 20.0748 7.00407 19.9999 7.00407C19.925 7.00407 19.8518 6.98191 19.7895 6.94038C19.7272 6.89885 19.6785 6.83981 19.6497 6.7707L19.4652 6.34625C19.1409 5.59538 18.5469 4.99334 17.8004 4.65894L17.2312 4.40472C17.1622 4.37296 17.1037 4.32206 17.0627 4.25806C17.0217 4.19406 16.9999 4.11965 16.9999 4.04364C16.9999 3.96763 17.0217 3.89322 17.0627 3.82922C17.1037 3.76522 17.1622 3.71432 17.2312 3.68256L17.7689 3.44334C18.5341 3.09941 19.1383 2.47511 19.457 1.69904L19.6475 1.24084C19.6753 1.16987 19.7239 1.10893 19.7869 1.06598C19.8499 1.02303 19.9244 1.00006 20.0007 1.00006C20.0769 1.00006 20.1514 1.02303 20.2144 1.06598C20.2774 1.10893 20.326 1.16987 20.3539 1.24084L20.5436 1.69829C20.8619 2.47451 21.4658 3.09908 22.2309 3.44334L22.7693 3.68331C22.8382 3.71516 22.8965 3.76605 22.9373 3.82997C22.9782 3.89389 22.9999 3.96816 22.9999 4.04402C22.9999 4.11987 22.9782 4.19414 22.9373 4.25806C22.8965 4.32198 22.8382 4.37287 22.7693 4.40472L22.1994 4.65819C21.4531 4.99293 20.8594 5.59523 20.5353 6.34625"
								fill="currentColor"
							/>
							<path
								d="M3 14V10"
								stroke="currentColor"
								strokeWidth={2}
								strokeLinecap="round"
							/>
							<path
								d="M21 14V10"
								stroke="currentColor"
								strokeWidth={2}
								strokeLinecap="round"
							/>
							<path
								d="M16.5 18V8"
								stroke="currentColor"
								strokeWidth={2}
								strokeLinecap="round"
							/>
							<path
								d="M12 22V2"
								stroke="currentColor"
								strokeWidth={2}
								strokeLinecap="round"
							/>
							<path
								d="M7.5 18V6"
								stroke="currentColor"
								strokeWidth={2}
								strokeLinecap="round"
							/>
						</svg>
						<p className="text-title">
							<span>I'm</span>
							<span>Listening...</span>
						</p>
					</div>
					<div className="container-chat">
						<div className="container-chat-limit">
							<div className="chats">
								<div
									className="chat-user"
									style={{ "--delay": 2 } as React.CSSProperties}
								>
									<p>
										<span style={{ "--word": 1 } as React.CSSProperties}>
											Where
										</span>
										<span style={{ "--word": 2 } as React.CSSProperties}>
											can
										</span>
										<span style={{ "--word": 3 } as React.CSSProperties}>
											i
										</span>
										<span style={{ "--word": 4 } as React.CSSProperties}>
											see
										</span>
										<span style={{ "--word": 5 } as React.CSSProperties}>
											examples
										</span>
										<span style={{ "--word": 6 } as React.CSSProperties}>
											of
										</span>
										<span style={{ "--word": 7 } as React.CSSProperties}>
											components
										</span>
										<span style={{ "--word": 8 } as React.CSSProperties}>
											for
										</span>
										<span style={{ "--word": 9 } as React.CSSProperties}>
											UI
										</span>
										<span style={{ "--word": 10 } as React.CSSProperties}>
											designs?
										</span>
									</p>
								</div>
								<div
									className="chat-ia"
									style={{ "--delay": 5 } as React.CSSProperties}
								>
									<p>
										<span style={{ "--word": 1 } as React.CSSProperties}>
											You
										</span>
										<span style={{ "--word": 2 } as React.CSSProperties}>
											have
										</span>
										<span style={{ "--word": 3 } as React.CSSProperties}>
											many
										</span>
										<span style={{ "--word": 4 } as React.CSSProperties}>
											good
										</span>
										<span style={{ "--word": 5 } as React.CSSProperties}>
											options
										</span>
										<span style={{ "--word": 6 } as React.CSSProperties}>
											depending
										</span>
										<span style={{ "--word": 7 } as React.CSSProperties}>
											on
										</span>
										<span style={{ "--word": 8 } as React.CSSProperties}>
											whether
										</span>
										<span style={{ "--word": 9 } as React.CSSProperties}>
											you
										</span>
										<span style={{ "--word": 10 } as React.CSSProperties}>
											want
										</span>
										<span style={{ "--word": 11 } as React.CSSProperties}>
											inspiration
										</span>
										<span style={{ "--word": 12 } as React.CSSProperties}>
											<strong>(designs)</strong>
										</span>
										<span style={{ "--word": 13 } as React.CSSProperties}>
											or
										</span>
										<span style={{ "--word": 14 } as React.CSSProperties}>
											ready-to-use
										</span>
										<span style={{ "--word": 15 } as React.CSSProperties}>
											code
										</span>
										<span style={{ "--word": 16 } as React.CSSProperties}>
											<strong>(components)</strong>.
										</span>
										<span style={{ "--word": 17 } as React.CSSProperties}>
											But
										</span>
										<span style={{ "--word": 18 } as React.CSSProperties}>
											<strong>UIVerse</strong>
										</span>
										<span style={{ "--word": 19 } as React.CSSProperties}>
											is
										</span>
										<span style={{ "--word": 20 } as React.CSSProperties}>
											a
										</span>
										<span style={{ "--word": 21 } as React.CSSProperties}>
											good
										</span>
										<span style={{ "--word": 22 } as React.CSSProperties}>
											option
										</span>
										<span style={{ "--word": 23 } as React.CSSProperties}>
											to
										</span>
										<span style={{ "--word": 24 } as React.CSSProperties}>
											start
										</span>
										<span style={{ "--word": 25 } as React.CSSProperties}>
											with.
										</span>
										<span style={{ "--word": 26 } as React.CSSProperties}>
											💪
										</span>
									</p>
								</div>
								<div
									className="chat-user"
									style={{ "--delay": 9 } as React.CSSProperties}
								>
									<p>
										<span style={{ "--word": 1 } as React.CSSProperties}>
											What's
										</span>
										<span style={{ "--word": 2 } as React.CSSProperties}>
											UIVerse?
										</span>
									</p>
								</div>
								<div
									className="chat-ia"
									style={{ "--delay": 11 } as React.CSSProperties}
								>
									<p>
										<span style={{ "--word": 1 } as React.CSSProperties}>
											It's
										</span>
										<span style={{ "--word": 2 } as React.CSSProperties}>
											a
										</span>
										<span style={{ "--word": 3 } as React.CSSProperties}>
											free
										</span>
										<span style={{ "--word": 4 } as React.CSSProperties}>
											gallery
										</span>
										<span style={{ "--word": 5 } as React.CSSProperties}>
											of
										</span>
										<span style={{ "--word": 6 } as React.CSSProperties}>
											UI
										</span>
										<span style={{ "--word": 7 } as React.CSSProperties}>
											components
										</span>
										<span style={{ "--word": 8 } as React.CSSProperties}>
											made
										</span>
										<span style={{ "--word": 9 } as React.CSSProperties}>
											only
										</span>
										<span style={{ "--word": 10 } as React.CSSProperties}>
											with
										</span>
										<span style={{ "--word": 11 } as React.CSSProperties}>
											HTML
										</span>
										<span style={{ "--word": 12 } as React.CSSProperties}>
											and
										</span>
										<span style={{ "--word": 13 } as React.CSSProperties}>
											CSS
										</span>
									</p>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</StyledWrapper>
	);
};

const StyledWrapper = styled.div`
  .container-vao {
    position: relative;
    width: 100%;
    height: 100%;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .input-orb:checked ~ .container-chat-ia {
    width: 300px;
    height: 300px;
    filter: blur(0px);
    opacity: 1;
  }

  .input-orb:checked ~ .orb {
    filter: drop-shadow(0 0 12px rgba(145, 71, 255, 0.3))
      drop-shadow(0 0 5px rgba(255, 0, 0, 0.3));
    transform-origin: center center;
    transform: translate(-50%, 110px);

    & .icons .svg {
      opacity: 1;
      filter: drop-shadow(0 0 4px #ffffff);
    }

    &:hover {
      transform: translate(-50%, 110px) scale(1.1);

      & .icons .svg .mic {
        opacity: 0;
        transform: scale(1.1);
        filter: drop-shadow(0 0 4px #ffffff);
      }

      & .icons .svg .close {
        transform: scale(1.1);
        filter: drop-shadow(0 0 4px #ffffff);
        opacity: 1;
      }
    }

    &:active {
      transform: translate(-50%, 110px) scale(0.9);
    }
  }

  .input-orb:not(:checked) ~ .container-chat-ia * {
    animation: none;
  }

  .input-orb:not(:checked) ~ .orb {
    filter: drop-shadow(0 0 4px rgba(255, 255, 255))
      drop-shadow(0 0 12px rgba(255, 255, 255))
      drop-shadow(0 0 12px rgba(145, 71, 255, 0.3))
      drop-shadow(0 0 5px rgba(255, 0, 0, 0.3));
    transform: scale(1.2) translate(-50%, -50%);

    & .ball {
      animation: circle2 4.2s ease-in-out infinite;
    }

    &:hover {
      transform: scale(1.4) translate(-50%, -50%);
      filter: drop-shadow(0 0 4px rgba(255, 255, 255))
        drop-shadow(0 0 8px rgba(255, 255, 255))
        drop-shadow(0 0 12px rgba(255, 255, 255))
        drop-shadow(0 0 10px rgba(145, 71, 255, 0.3))
        drop-shadow(0 6px 26px rgba(255, 0, 0, 0.3));

      & .icons .svg {
        transform: scale(1.1);
        filter: drop-shadow(0 0 4px #ffffff);
        opacity: 1;
      }
    }

    &:active {
      transform: scale(1.2) translate(-50%, -50%);
      filter: drop-shadow(0 0 4px rgba(255, 255, 255))
        drop-shadow(0 0 8px rgba(255, 255, 255))
        drop-shadow(0 0 12px rgba(255, 255, 255))
        drop-shadow(0 0 10px rgba(145, 71, 255, 0.3))
        drop-shadow(0 6px 26px rgba(255, 0, 0, 0.3));
    }

    & * {
      animation: none;
    }
  }

  @keyframes circle2 {
    0% {
      transform: scale(1.5);
    }

    15% {
      transform: scale(1.53);
    }

    30% {
      transform: scale(1.48);
    }

    45% {
      transform: scale(1.44);
    }

    60% {
      transform: scale(1.47);
    }

    85% {
      transform: scale(1.53);
    }

    100% {
      transform: scale(1.5);
    }
  }

  .container-chat-ia {
    opacity: 0;
    filter: blur(50px);
    display: flex;
    flex-direction: column;
    width: 64px;
    height: 64px;
    padding: 0.5rem;
    border-radius: 2rem;
    box-shadow:
      6px 6px 12px rgba(255, 0, 2, 0.1),
      -6px 6px 12px rgba(59, 130, 246, 0.1);
    gap: 6px;
    transition: all 0.6s cubic-bezier(0.175, 0.885, 0.32, 1.1);
  }

  .container-title {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.5rem;
    gap: 6px;

    & svg {
      color: #ff0002;
      animation: animation-color-svg 8s 1s infinite both;
    }

    & .text-title {
      font-size: 14px;
      font-weight: 500;
      background-image: linear-gradient(
        to left,
        #ff0002 0% 20%,
        #3b82f6 50%,
        #ff0002 80% 100%
      );
      background-clip: text;
      -webkit-background-clip: text;
      color: transparent;
      background-size: 800px;
      animation: animation-color-text 8s infinite linear;
    }
  }

  @keyframes animation-color-svg {
    0%,
    30% {
      color: #ff0002;
    }

    15% {
      color: #3b82f6;
    }
  }

  @keyframes animation-color-text {
    0% {
      background-position: -800px;
    }

    50% {
      background-position: 0px;
    }
  }

  @keyframes animation-points {
    0% {
      transform: translateY(0);
    }

    50% {
      transform: translateY(-15px);
    }
  }

  .container-chat {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    font-size: 13px;
    background-image: linear-gradient(
      to top left,
      rgb(255, 0, 2, 0.22),
      rgb(59, 130, 246, 0.22)
    );
    border-radius: 1.5rem;
    overflow: hidden;

    &::after {
      position: absolute;
      content: "";
      inset: 0;
      background: repeating-conic-gradient(
          rgba(255, 255, 255, 0.2) 0.0000001%,
          rgb(232, 232, 232, 0.8) 0.000104%
        )
        60% 60%/600% 600%;
    }
  }

  .container-chat-limit {
    display: flex;
    -webkit-mask: linear-gradient(0deg, white 85%, transparent 95% 100%);
    mask: linear-gradient(0deg, white 85%, transparent 95% 100%);
    z-index: 999;
  }

  .chats {
    display: flex;
    flex-direction: column;
    padding: 2rem 1rem 1rem 1rem;
    animation: animation-chats 16s both ease;
    gap: 0.25rem;
  }

  .chat-user {
    display: flex;
    justify-content: end;

    & p {
      opacity: 0;
      transform: translateY(10px);
      width: 85%;
      display: flex;
      justify-content: center;
      flex-wrap: wrap;
      gap: 0.25rem;
      line-height: 1;
      padding: 0.625rem;
      color: #898989;
      border-radius: 0.625rem 0.625rem 0 0.625rem;
      background-color: rgba(255, 255, 255, 0.5);
      animation: animation-chat 1s calc(var(--delay) * 1s) both
        cubic-bezier(0.175, 0.885, 0.32, 1.275);

      & span {
        opacity: 0;
        transform: translateY(10px);
        display: block;
        animation: animation-chat 1s calc(var(--delay) * 1s + var(--word) * 0.1s)
          both cubic-bezier(0.175, 0.885, 0.32, 1.275);
      }
    }
  }

  .chat-ia {
    display: flex;

    & p {
      opacity: 0;
      transform: translateY(10px);
      width: 85%;
      display: flex;
      flex-wrap: wrap;
      gap: 0.25rem;
      line-height: 1;
      padding: 0.625rem 0;
      color: #393754;
      animation: animation-chat 1s calc(var(--delay) * 1s) both
        cubic-bezier(0.175, 0.885, 0.32, 1.275);

      & span {
        opacity: 0;
        transform: translateY(10px);
        display: block;
        animation: animation-chat 1s calc(var(--delay) * 1s + var(--word) * 0.1s)
          both cubic-bezier(0.175, 0.885, 0.32, 1.275);
      }
    }
  }

  @keyframes animation-chat {
    100% {
      opacity: 1;
      transform: translateY(0px);
    }
  }

  @keyframes animation-chats {
    0%,
    55% {
      transform: translateY(0px);
    }

    70% {
      transform: translateY(-70px);
    }

    80%,
    100% {
      transform: translateY(-110px);
    }
  }

  .orb {
    position: absolute;
    left: 50%;
    top: 50%;
    transform-origin: left top;
    transform: translate(-50%, -50%);
    width: 64px;
    height: 64px;
    display: flex;
    transition: all 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    cursor: pointer;
    z-index: 999999;

    & .icons .svg .close {
      opacity: 0;
    }
  }

  .icons {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: #ffffff;
    display: flex;
    flex-direction: column;
    transition: all 0.3s ease-in-out;
    z-index: 999;

    & .svg {
      width: 24px;
      height: 24px;
      flex-shrink: 0;
      opacity: 0.5;
      transition: all 0.3s ease-in-out;
    }
  }

  .ball {
    display: flex;
    width: 64px;
    height: 64px;
    flex-shrink: 0;
    border-radius: 50px;
    background-color: #ff0002;
    filter: url(#gooey);
  }

  .container-lines {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 100px;
    height: 100px;
    background-image: radial-gradient(
      ellipse at center,
      rgba(255, 255, 255, 0.75) 15%,
      #3b82f6 50%
    );
    clip-path: polygon(
      50% 25%,
      65% 30%,
      75% 42%,
      75% 58%,
      65% 70%,
      50% 75%,
      35% 70%,
      26% 58%,
      25% 42%,
      35% 30%
    );
    animation: animation-ball 15s both ease;
    pointer-events: none;
  }

  @keyframes animation-ball {
    2% {
      clip-path: polygon(
        50% 25%,
        50% 0,
        75% 42%,
        75% 58%,
        65% 70%,
        50% 75%,
        35% 70%,
        26% 58%,
        25% 42%,
        50% 0
      );
    }

    4% {
      clip-path: polygon(
        50% 25%,
        70% 0,
        75% 42%,
        85% 66%,
        65% 100%,
        50% 75%,
        35% 100%,
        15% 65%,
        25% 42%,
        30% 0
      );
    }

    6% {
      clip-path: polygon(
        50% 25%,
        50% 15%,
        75% 42%,
        75% 58%,
        65% 70%,
        50% 75%,
        35% 70%,
        26% 58%,
        25% 42%,
        50% 15%
      );
    }

    7%,
    59% {
      clip-path: polygon(
        50% 25%,
        100% 12%,
        75% 42%,
        85% 66%,
        65% 70%,
        50% 75%,
        35% 70%,
        15% 65%,
        25% 42%,
        0 12%
      );
    }

    9%,
    57% {
      clip-path: polygon(
        50% 25%,
        50% 0,
        75% 42%,
        75% 58%,
        65% 70%,
        50% 75%,
        35% 70%,
        26% 58%,
        25% 42%,
        50% 0
      );
    }

    12%,
    55%,
    61% {
      clip-path: polygon(
        50% 25%,
        65% 30%,
        75% 42%,
        75% 58%,
        65% 70%,
        50% 75%,
        35% 70%,
        26% 58%,
        25% 42%,
        35% 30%
      );
    }
  }

  .container-borders {
    position: relative;
    display: flex;
    width: 56px;
    height: 56px;
    margin: 4px;
    border-radius: 50px;
    border-top: 8px solid #3b82f6;
    border-left: 8px solid #3b82f6;
  }

  .container-rings {
    aspect-ratio: 1;
    border-radius: 50%;
    position: absolute;
    inset: 0;
    perspective: 11rem;

    &:before,
    &:after {
      content: "";
      position: absolute;
      inset: 0;
      background: rgba(255, 0, 0, 1);
      border-radius: 50%;
      border: 6px solid transparent;
      mask:
        linear-gradient(#fff 0 0) padding-box,
        linear-gradient(#fff 0 0);
      background: linear-gradient(white, blue, magenta, violet, lightyellow)
        border-box;
      mask-composite: exclude;
    }
  }

  .container-rings::before {
    animation: ring180 10s linear infinite;
  }

  .container-rings::after {
    animation: ring90 10s linear infinite;
  }

  @keyframes ring180 {
    0% {
      transform: rotateY(180deg) rotateX(180deg) rotateZ(180deg);
    }

    50% {
      transform: rotateY(360deg) rotateX(360deg) rotateZ(360deg) scale(1.1);
    }

    100% {
      transform: rotateY(540deg) rotateX(540deg) rotateZ(540deg);
    }
  }

  @keyframes ring90 {
    0% {
      transform: rotateY(90deg) rotateX(90deg) rotateZ(90deg);
    }

    50% {
      transform: rotateY(270deg) rotateX(270deg) rotateZ(270deg) scale(1.1);
    }

    100% {
      transform: rotateY(450deg) rotateX(450deg) rotateZ(450deg);
    }
  }`;

export default Button;
