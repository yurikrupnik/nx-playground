# Done
FROM nginx:alpine AS nginx
WORKDIR /app
ARG DIST_PATH
RUN test -n "$DIST_PATH" || (echo "DIST_PATH  not set" && false)
COPY ./$DIST_PATH /usr/share/nginx/html
ENV PORT=80
EXPOSE ${PORT}
CMD ["nginx", "-g", "daemon off;"]
